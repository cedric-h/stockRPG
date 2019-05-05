#![allow(clippy::many_single_char_names)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(try_from)]

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

mod assemblages;
mod camera;
mod compendium;
mod components;
mod dev_ui;
mod dyon;
mod hal_state;
mod image_bundle;
mod local_state;
mod phys_state;
mod user_input;
mod winit_state;

mod prelude;
use crate::prelude::*;

use specs::prelude::*;
use specs::{
    DispatcherBuilder, Entities, LazyUpdate, ReadExpect, ReadStorage, System, World, WriteExpect,
    WriteStorage,
};

struct AddHitboxesToPhys;
impl<'a> System<'a> for AddHitboxesToPhys {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, PhysState>,
        WriteStorage<'a, Hitbox>,
        WriteStorage<'a, Phys>,
    );

    fn run(&mut self, (entities, mut physics_state, mut hitboxes, mut physes): Self::SystemData) {
        use specs::Join;

        for (ent, mut hitbox) in (&*entities, hitboxes.drain()).join() {
            //get a handle to the body for the hitbox
            let phys_comp = physics_state.phys_from_hitbox(&mut hitbox);

            //add the entity as some user data so that when we find it via raycasting
            //we can detect it for what it is.
            physics_state.name_as_ent(&phys_comp, Box::new(ent));

            //add the new physics body to the world.
            physes.insert(ent, phys_comp).unwrap();
        }
    }
}

struct BuildAppearances;
impl<'a> System<'a> for BuildAppearances {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, ImageBundle>,
        WriteStorage<'a, Appearance>,
        WriteStorage<'a, AppearanceBuilder>,
    );

    fn run(
        &mut self,
        (entities, image_bundle, mut appears, mut appear_builders): Self::SystemData,
    ) {
        use arraytools::ArrayTools;
        use specs::Join;

        //for every AppearanceBuilder in the world, delete it, but then do
        for (ent, mut appear_builder) in (&*entities, &mut appear_builders).join() {
            if !appear_builder.built {
                appear_builder.built = true;
                if let Some(base_u32) = image_bundle.map.get(&appear_builder.image_name) {
                    let base = base_u32.map(|x| x as f32);
                    let conf = appear_builder.uv_override;
                    appears
                        .insert(
                            ent,
                            Appearance {
                                uv_rect: [
                                    (base[0]) + conf[0],
                                    (base[1]) + conf[1],
                                    if conf[2] == 0.0 { base[2] } else { conf[2] },
                                    if conf[3] == 0.0 { base[3] } else { conf[3] },
                                ],
                            },
                        )
                        .unwrap();
                } else {
                    error!(
                        "uv indexes not found for image name: {}",
                        appear_builder.image_name
                    )
                }
            }
        }
    }
}

struct Exploding;
impl<'a> System<'a> for Exploding {
    type SystemData = (
        ReadExpect<'a, LocalState>,
        ReadExpect<'a, Assemblager>,
        ReadExpect<'a, LazyUpdate>,
        ReadExpect<'a, PhysState>,
        Entities<'a>,
        ReadStorage<'a, Phys>,
        WriteStorage<'a, Explodeable>,
    );

    fn run(&mut self, (ls, assemblager, lu, ps, ents, physes, mut explodeables): Self::SystemData) {
        use specs::Join;
        use winit::VirtualKeyCode::B;

        if ls.tapped_keys.contains(&B) {
            info!("kerboom!");
            (&explodeables, &ents, &physes)
                .join()
                .filter_map(|(explo, ent, phys)| {
                    let pos = *ps.location(phys).unwrap();

                    for _ in 0..explo.chunks_count {
                        let which_gib: i32 = OsRng::new().unwrap().gen_range(1, 10);
                        let gib_ent = assemblager.build_at("melon gib", &lu, &ents, pos);

                        lu.insert(
                            gib_ent,
                            AppearanceBuilder {
                                image_name: format!("melon_gib_{:?}", which_gib),
                                ..AppearanceBuilder::default()
                            },
                        );

                        lu.insert(
                            gib_ent,
                            ApplyForce {
                                vec: ApplyForce::random_2d_vec() * explo.force.vec.x,
                                ..explo.force
                            },
                        );
                    }

                    if explo.delete_entity {
                        ents.delete(ent).unwrap();
                    }

                    //there's no point in removing the component if it was just removed when the
                    //entity was deleted.
                    (explo.delete_component && !explo.delete_entity).as_some(ent)
                })
                //the collect and iter serve to make sure explodeables is dropped, so that it can
                //then be used to remove the explodeable components we'd like to get rid of.
                .collect::<Vec<_>>()
                .iter()
                .for_each(|ent| {
                    explodeables.remove(*ent);
                });
        }
    }
}

struct ApplyForces;
impl<'a> System<'a> for ApplyForces {
    type SystemData = (
        WriteExpect<'a, PhysState>,
        ReadExpect<'a, LocalState>,
        Entities<'a>,
        WriteStorage<'a, ApplyForce>,
        ReadStorage<'a, Phys>,
    );

    fn run(&mut self, (mut ps, ls, ents, mut forces, physes): Self::SystemData) {
        use nphysics3d::{
            math::{Force, ForceType},
            object::Body,
        };
        use specs::Join;
        use std::f32;

        (&mut forces, &physes, &ents)
            .join()
            .filter_map(|(force, phys, ent)| {
                force.time_elapsed += ls.last_frame_duration;

                if force.time_elapsed >= force.duration {
                    Some(ent)
                } else {
                    ps.rigid_body_mut(phys).unwrap().apply_force(
                        0,
                        &Force::linear(
                            force.vec * f32::consts::E.powf(force.time_elapsed * force.decay),
                        ),
                        ForceType::Impulse,
                        true,
                    );
                    None
                }
            })
            //the collect and iter serve to make sure forces is dropped, so that it can be used
            //to remove the force components we'd like to get rid of.
            .collect::<Vec<_>>()
            .iter()
            .for_each(|ent| {
                forces.remove(*ent);
            });
    }
}

const GRABBING_RANGE: f32 = 5.0;
struct Interact;
impl<'a> System<'a> for Interact {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, LocalState>,
        ReadExpect<'a, PhysState>,
        ReadStorage<'a, Phys>,
        ReadStorage<'a, Interactable>,
        ReadStorage<'a, MovementControls>,
        WriteStorage<'a, ScriptEvent>,
    );

    fn run(
        &mut self,
        (ents, local_state, ps, physes, interactables, movement_controls, mut script_events): Self::SystemData,
    ) {
        use specs::Join;
        use winit::VirtualKeyCode::E;

        //minimum distance the interactable must be at to be interacted with
        if local_state.tapped_keys.contains(&E) {
            //grab the player's x and y coordinates from the physics state
            let player_pos = {
                let (phys, _) = (&physes, &movement_controls).join().next().unwrap();
                ps.location(phys).unwrap().xy()
            };

            let closest_interactable: Option<(&Interactable, specs::Entity)> =
                (&physes, &interactables, &ents)
                    .join()
                    //turn each (phys, interactable) into (distance, interactable)
                    .map(|(phys, interactable, ent)| {
                        let interactable_pos = ps.location(phys).unwrap().xy();
                        (
                            glm::distance2(&player_pos, &interactable_pos),
                            interactable,
                            ent,
                        )
                    })
                    //find the tuple with the smallest distance from the player
                    .fold(
                        (GRABBING_RANGE, None),
                        |(closest, x), (distance, interactable, ent)| {
                            if distance < closest {
                                //if this tuple's distance is smaller than the accumulator's
                                //return it from the closure so it's acc for the next iterations.
                                (distance, Some((interactable, ent)))
                            } else {
                                //if this tuple's distance isn't smaller,
                                //return the current accumulator so it stays the same.
                                (closest, x)
                            }
                        },
                    )
                    //turn it from a (distance, (interactable, ent)) into (interactable, ent)
                    .1;

            //if an interactable was close enough, log it's message and launch the scripting event
            if let Some((Interactable { message, script }, ent)) = closest_interactable {
                println!("{}", &message);
                script_events.insert(ent, script.clone()).unwrap();
            }
        }
    }
}

struct KeyboardMovementControls;
impl<'a> System<'a> for KeyboardMovementControls {
    type SystemData = (
        ReadStorage<'a, MovementControls>, //so you know who's moving
        ReadStorage<'a, Phys>,             //because they'll need a physical representation to move.
        ReadExpect<'a, LocalState>, //because you'll need somewhere to pull the movement info from.
        WriteExpect<'a, PhysState>, //because you're moving their position in the physical world
    );

    fn run(&mut self, (movs, physes, local_state, mut ps): Self::SystemData) {
        use nphysics3d::{
            math::{Force, ForceType},
            object::Body,
        };

        use winit::VirtualKeyCode;
        let keys = &local_state.last_input.keys_held;
        if !keys.contains(&VirtualKeyCode::LControl) {
            let vertical = glm::vec3(0.0, 1.0, 0.0);
            let horizontal = glm::vec3(-1.0, 0.0, 0.0);
            let move_vector = keys
                .iter()
                .fold(glm::make_vec3(&[0.0, 0.0, 0.0]), |vec, key| match *key {
                    VirtualKeyCode::A => vec + horizontal,
                    VirtualKeyCode::D => vec - horizontal,
                    VirtualKeyCode::S => vec + vertical,
                    VirtualKeyCode::W => vec - vertical,
                    _ => vec,
                });
            if move_vector != glm::zero() {
                use specs::Join;

                for (phys, mov) in (&physes, &movs).join() {
                    let body = ps.rigid_body_mut(phys).unwrap();

                    body.apply_force(
                        0,
                        &Force::linear(
                            move_vector.normalize() * mov.speed - body.velocity().linear,
                        ),
                        ForceType::Force,
                        true,
                    );
                }
            }
        }
    }
}

struct EditorPlaceControls;
impl<'a> System<'a> for EditorPlaceControls {
    type SystemData = (
        ReadStorage<'a, Phys>,
        ReadExpect<'a, LocalState>,
        Entities<'a>,
        WriteExpect<'a, PhysState>,
        WriteExpect<'a, Compendium>,
        ReadExpect<'a, Assemblager>,
        ReadStorage<'a, Assemblaged>,
    );

    fn run(
        &mut self,
        (physes, local_state, entities, mut ps, mut compium, asmblgr, asmblgd): Self::SystemData,
    ) {
        let mouse_clicked_this_frame = local_state.last_input.mouse_state.unwrap_or(false);
        let new_mouse = &local_state.last_input.mouse_pos;

        if let Some(ent) = compium.place_me_entity {
            if let Some(phys) = physes.get(ent) {
                //we could get local_state.mouse_pos, but that's simply the last known mouse_pos.
                //we want the last_input one, since that'll tell us whether or not they moved the mouse
                //this frame; that'll let us only move the thing when we really need to.
                if new_mouse.is_some() || mouse_clicked_this_frame {
                    //get collision pos
                    let mouse_pos = new_mouse.unwrap_or(local_state.mouse_pos);
                    let raycaster = Raycaster::point_from_camera(&mouse_pos, &local_state);
                    let ground_collision_pos = raycaster.cast_to_ground_pos(&ps).unwrap();

                    //get its offset recorded in the type editor
                    let Assemblaged { built_from } = asmblgd.get(ent).unwrap();
                    let hitbox = asmblgr.assemblages[built_from]
                        .iter()
                        .find(|x| x.name() == "Hitbox")
                        .unwrap()
                        .downcast_ref::<Hitbox>()
                        .unwrap();

                    //set the location to the combination of the two
                    ps.set_location(&phys, &(ground_collision_pos + hitbox.position));
                }

                if mouse_clicked_this_frame {
                    compium.place_me_entity = None;
                }
            }
        }
        //if we don't have anything to place, but they've clicked,
        //they're probably trying to select something.
        else if mouse_clicked_this_frame {
            let raycaster = Raycaster::point_from_camera(&local_state.mouse_pos, &local_state);
            let clicked_body_handle = ps
                .world
                .collider_world()
                .interferences_with_ray(&raycaster.ray, &raycaster.collision_group)
                .next()
                .unwrap()
                .0 // it's (collider, rayhit), we want collider.body()
                .body();

            if let Ok(id) =
                serde_json::from_str(&ps.world.body(clicked_body_handle).unwrap().name())
            {
                compium.chosen_entity = Some(entities.entity(id));
            }
        }
    }
}

struct EditorSave;
impl<'a> System<'a> for EditorSave {
    type SystemData = (
        ReadExpect<'a, Assemblager>,
        ReadExpect<'a, LazyUpdate>,
        ReadExpect<'a, LocalState>,
    );

    fn run(&mut self, (asmblgr, lu, ls): Self::SystemData) {
        use winit::VirtualKeyCode::{LControl, S};
        if ls.last_input.keys_held.contains(&LControl) && ls.tapped_keys.contains(&S) {
            asmblgr.save_json(&lu);
        }
    }
}

#[derive(Default)]
struct PhysicsUpdate {
    pub reader_id: Option<specs::ReaderId<specs::storage::ComponentEvent>>,
}
impl<'a> System<'a> for PhysicsUpdate {
    type SystemData = (WriteExpect<'a, PhysState>, ReadStorage<'a, Phys>);

    fn setup(&mut self, res: &mut specs::Resources) {
        Self::SystemData::setup(res);
        self.reader_id = Some(WriteStorage::<Phys>::fetch(&res).register_reader());
    }

    fn run(&mut self, (mut ps, physes): Self::SystemData) {
        use nphysics3d::{
            math::{Force, ForceType},
            object::{Body, BodyHandle},
        };
        use specs::{storage::ComponentEvent, Join};

        for event in physes
            .channel()
            .read(self.reader_id.as_mut().expect("ReaderId not found"))
        {
            match event {
                ComponentEvent::Removed(id) => {
                    let id_string = id.to_string();
                    let handles = ps
                        .world
                        .bodies_with_name(&id_string)
                        .map(|x| x.handle())
                        .collect::<Vec<BodyHandle>>();
                    ps.world.remove_bodies(&handles);
                }
                _ => (),
            }
        }

        for handle in (&physes).join() {
            let timestep = ps.world.timestep();
            let body = ps.rigid_body_mut(handle).unwrap();

            let lv = &body.velocity().linear;
            //perhaps replace that 0.5_f32 with a fraction of the actual velocity.
            let force = 2.0_f32.min(body.augmented_mass().linear * glm::length(lv) / timestep);

            if force != 0.0 {
                body.apply_force(
                    0,
                    &Force::linear(-lv.normalize() * force),
                    ForceType::Force,
                    true,
                );
            }
        }

        ps.world.step();
    }
}

struct SpriteSheetAnimate;
impl<'a> System<'a> for SpriteSheetAnimate {
    type SystemData = (
        WriteStorage<'a, Appearance>,
        ReadStorage<'a, Animation>,
        ReadExpect<'a, LocalState>,
    );

    fn run(&mut self, (mut appearances, animations, local_state): Self::SystemData) {
        use specs::Join;

        for (app, ani) in (&mut appearances, &animations).join() {
            let frame_index =
                (local_state.elapsed_time * ani.fps).floor() % (ani.frame_count as f32);

            app.uv_rect[0] = app.uv_rect[2] * frame_index;
        }
    }
}

struct Render;
impl<'a> System<'a> for Render {
    type SystemData = (
        ReadStorage<'a, Phys>,
        ReadStorage<'a, CameraFocus>,
        ReadStorage<'a, Appearance>,
        WriteExpect<'a, HalState>,
        ReadExpect<'a, LocalState>,
        ReadExpect<'a, PhysState>,
    );

    fn run(
        &mut self,
        (physes, camera_focuses, appearances, mut hal_state, local_state, ps): Self::SystemData,
    ) {
        use specs::Join;

        let fill = camera_focuses
            .join()
            .next()
            .map(|cf| cf.background_color)
            .unwrap_or([0.1, 0.2, 0.3, 1.0]);

        let projection = if local_state.is_orthographic {
            local_state.orthographic_projection
        } else {
            local_state.perspective_projection
        };
        let view_projection = projection * local_state.camera.view_matrix;
        if let Err(e) = hal_state.draw_appearances_frame(
            &view_projection,
            &(&appearances, &physes)
                .join()
                .map(|(app, phys)| (app, ps.rigid_body(phys).unwrap().position()))
                .collect::<Vec<_>>(),
            fill,
        ) {
            panic!("Rendering Error: {:?}", e);
        }
    }
}

struct Input {
    winit_state: WinitState,
}
impl<'a> System<'a> for Input {
    type SystemData = WriteExpect<'a, LocalState>;

    fn run(&mut self, mut local_state: Self::SystemData) {
        let inputs = UserInput::poll_events_loop(&mut self.winit_state);
        /* if inputs.new_frame_size.is_some() {
            debug!("Window changed size, changing HalState...");
            hal_state.resize_swapchain();
        }*/
        local_state.update_from_input(inputs);
    }
}

struct CameraLerp;
impl<'a> System<'a> for CameraLerp {
    type SystemData = (
        WriteExpect<'a, LocalState>,
        ReadExpect<'a, PhysState>,
        ReadStorage<'a, Phys>,
        ReadStorage<'a, CameraFocus>,
    );

    fn run(&mut self, (mut local_state, ps, physes, focuses): Self::SystemData) {
        use specs::Join;
        let dur = local_state.last_frame_duration;

        for (phys, foc) in (&physes, &focuses).join() {
            local_state.camera.lerp_towards(
                ps.location(phys).unwrap() + glm::vec3(-0.5, 0.0, 0.0),
                dur * foc.interpolation_speed,
            );
        }
    }
}

fn main() {
    use current::CurrentGuard;

    simple_logger::init().unwrap();

    //-- Specs Resources:
    //Developer Tools stuff
    let mut dev_ui = DevUiUpdate::new();
    let compendium = Compendium::new();
    let mut dyon_state = DyonState::new();
    let dyon_console = DyonConsole::default();
    //spritesheet texture indexes
    let image_bundle = ImageBundle::new();
    //windowing stuff
    let winit_state = WinitState::default();
    let mut local_state = LocalState::from_winit_state(&winit_state);
    let hal_state = HalState::new(&winit_state.window).unwrap_or_else(|e| panic!(e));
    //physics
    let physics_state = PhysState::new();

    let mut world = World::new();
    #[rustfmt::skip]
    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(Input { winit_state })
        .with(AddHitboxesToPhys,            "hitboxes to phys",     &[])
        .with(ApplyForces,                  "apply forces",         &["hitboxes to phys"])
        .with(PhysicsUpdate::default(),     "physics update",       &["apply forces"])
        .with(CameraLerp,                   "lerp camera",          &["physics update"])
        .with(Interact,                     "player interact",      &["physics update"])
        .with(KeyboardMovementControls,     "keyboard controls",    &["physics update"])
        .with(EditorPlaceControls,          "editor place",         &["physics update"])
        .with(EditorSave,                   "save world to file",   &["physics update"])
        .with(Exploding,                    "explode effect",       &["physics update"])
        .with(BuildAppearances,             "builders to appears",  &[])
        .with(SpriteSheetAnimate,           "animate",              &["builders to appears"])
        .with(Render,                       "render",               &["animate"])
        .build();

    dispatcher.setup(&mut world.res);

    let mut assemblager = Assemblager::new();

    macro_rules! register {
         ($($name:ty),+ $(,)?) => {$(
            world.register::<$name>();
            assemblager.register_component(<$name>::default());
         )+}
    };

    world.register::<Assemblaged>();
    world.register::<Appearance>();
    world.register::<Phys>();
    register!(
        AppearanceBuilder,
        MovementControls,
        Interactable,
        ScriptEvent,
        Explodeable,
        CameraFocus,
        Animation,
        Hitbox,
    );

    assemblager.load_save(&mut world);
    local_state.find_camera_focus_and_zoom(&world);

    world.add_resource(compendium);
    world.add_resource(image_bundle);
    world.add_resource(hal_state);
    world.add_resource(local_state);
    world.add_resource(physics_state);
    world.add_resource(assemblager);
    world.add_resource(dyon_console);

    while !world.read_resource::<LocalState>().quit {
        //your everyday ECS systems are run first
        dispatcher.dispatch(&mut world.res);

        //add anything the systems want to lazily add
        world.maintain();

        //the scripting system completely breaks ECS
        let specs_world_guard = CurrentGuard::new(&mut world);
        dyon_state.run();
        drop(specs_world_guard);

        //next, the developer UI shows us what happened
        dev_ui.run(&world);
    }
}
