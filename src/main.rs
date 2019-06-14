#![allow(clippy::many_single_char_names)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

mod assemblages;
mod camera;
mod compendium;
mod comps;
mod dev_ui;
mod dyon;
mod glium;
mod glutin_state;
mod image_bundle;
mod local_state;
mod phys_state;
mod user_input;
//mod wgpu;
//mod winit_state;

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

    fn run(&mut self, (ents, mut physics_state, mut hitboxes, mut physes): Self::SystemData) {
        for (ent, mut hitbox) in (&*ents, hitboxes.drain()).join() {
            // get a handle to the body for the hitbox
            let phys_comp = physics_state.phys_from_hitbox(&mut hitbox);

            // add the entity as some user data so that when we find it via raycasting
            // we can detect it for what it is.
            physics_state.name_as_ent(&phys_comp, Box::new(ent));

            // add the new physics body to the world.
            physes.insert(ent, phys_comp).unwrap();
        }
    }
}

struct BuildAppearances;
impl<'a> System<'a> for BuildAppearances {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, ImageBundle>,
        ReadExpect<'a, SpritesheetDimensions>,
        WriteStorage<'a, Appearance>,
        WriteStorage<'a, AppearanceBuilder>,
    );

    fn run(
        &mut self,
        (ents, image_bundle, sss, mut appears, mut appear_builders): Self::SystemData,
    ) {
        use arraytools::ArrayTools;

        // for every AppearanceBuilder in the world,
        for (ent, mut appear_builder) in (&*ents, &mut appear_builders).join() {
            if !appear_builder.built {
                appear_builder.built = true;
                if let Some(base_u32) = image_bundle.map.get(&appear_builder.image_name) {
                    let base = base_u32.map(|x| x as f32);
                    let size = appear_builder.size_override;
                    let adj = appear_builder.uv_adjust;
                    let size = [
                        if size[0] == 0.0 { base[2] } else { size[0] },
                        if size[1] == 0.0 { base[3] } else { size[1] },
                    ];
                    let start = [base[0] + adj[0], base[1] + adj[1]];
                    appears
                        .insert(
                            ent,
                            Appearance {
                                // uv coordinates are relative to the spritesheet size.
                                uvs: [
                                    start[0] / sss.x,
                                    start[1] / sss.y,
                                    (start[0] + size[0]) / sss.x,
                                    (start[1] + size[1]) / sss.y,
                                ],
                                // the size of things is based on how many pixels they have.
                                size: size.map(|x| x / 64.0),
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
        use glutin::VirtualKeyCode::B;

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

                    // there's no point in removing the component if it was just removed when the
                    // entity was deleted.
                    (explo.delete_component && !explo.delete_entity).as_some(ent)
                })
                // the collect and iter serve to make sure explodeables is dropped, so that it can
                // then be used to remove the explodeable components we'd like to get rid of.
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
            // the collect and iter serve to make sure forces is dropped, so that it can be used
            // to remove the force components we'd like to get rid of.
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
        use glutin::VirtualKeyCode::E;

        // minimum distance the interactable must be at to be interacted with
        if local_state.tapped_keys.contains(&E) {
            // grab the player's x and y coordinates from the physics state
            let (player_pos, player_ent) = {
                let (phys, _, ent) = (&physes, &movement_controls, &ents).join().next().unwrap();
                (ps.location(phys).unwrap().xy(), ent.id().clone())
            };

            let closest_interactable: Option<(&Interactable, specs::Entity)> =
                (&physes, &interactables, &ents)
                    .join()
                    // turn each (phys, interactable) into (distance, interactable)
                    .map(|(phys, interactable, ent)| {
                        let interactable_pos = ps.location(phys).unwrap().xy();
                        (
                            glm::distance2(&player_pos, &interactable_pos),
                            interactable,
                            ent,
                        )
                    })
                    // find the tuple with the smallest distance from the player
                    .fold(
                        (GRABBING_RANGE, None),
                        |(closest, x), (distance, interactable, ent)| {
                            if distance < closest {
                                // if this tuple's distance is smaller than the accumulator's
                                // return it from the closure so it's acc for the next iterations.
                                (distance, Some((interactable, ent)))
                            } else {
                                // if this tuple's distance isn't smaller,
                                // return the current accumulator so it stays the same.
                                (closest, x)
                            }
                        },
                    )
                    // turn it from a (distance, (interactable, ent)) into (interactable, ent)
                    .1;

            // if an interactable was close enough, log it's message and launch the scripting event
            if let Some((Interactable { script }, ent)) = closest_interactable {
                script_events
                    .insert(ent, script.clone_with_payload(player_ent))
                    .unwrap();
            }
        }
    }
}

struct KeyboardMovementControls;
impl<'a> System<'a> for KeyboardMovementControls {
    type SystemData = (
        ReadStorage<'a, MovementControls>, // so you know who's moving
        ReadStorage<'a, Phys>, // because they'll need a physical representation to move.
        ReadExpect<'a, LocalState>, // because you'll need somewhere to pull the movement info from.
        WriteExpect<'a, PhysState>, // because you're moving their position in the physical world
    );

    fn run(&mut self, (movs, physes, local_state, mut ps): Self::SystemData) {
        use nphysics3d::{
            math::{Force, ForceType},
            object::Body,
        };

        use glutin::VirtualKeyCode;
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
        WriteStorage<'a, BoxOutline>,
        WriteExpect<'a, PhysState>,
        WriteExpect<'a, Compendium>,
        ReadStorage<'a, Phys>,
        ReadStorage<'a, Assemblaged>,
        Entities<'a>,
        ReadExpect<'a, LocalState>,
        ReadExpect<'a, Assemblager>,
    );

    fn run(
        &mut self,
        (mut outlines, mut ps, mut compium, physes, asmblgd, ents, ls, asmblgr): Self::SystemData,
    ) {
        if !compium.show_dev_ui {
            return;
        }

        use glutin::VirtualKeyCode::G;
        let mouse_clicked_this_frame = ls.last_input.mouse_state.unwrap_or(false);
        let new_mouse = &ls.last_input.mouse_pos;

        if let Some(ent) = compium.get_chosen_ent() {
            //toggle mouselock when they press G
            if ls.tapped_keys.contains(&G) {
                compium.mouselock_chosen_ent = !compium.mouselock_chosen_ent;
            }

            if compium.mouselock_chosen_ent {
                if let Some(phys) = physes.get(ent) {
                    // we could get ls.mouse_pos, but that's simply the last known mouse_pos.
                    // we want the last_input one, since that'll tell us whether or not they
                    // moved the mouse this frame; that'll let us only move the thing when we
                    // really need to.
                    if new_mouse.is_some() || mouse_clicked_this_frame {
                        // get collision pos
                        let mouse_pos = new_mouse.unwrap_or(ls.mouse_pos);
                        let raycaster = Raycaster::point_from_camera(&mouse_pos, &ls);
                        let ground_collision_pos = raycaster.cast_to_ground_pos(&ps).unwrap();

                        // get its offset recorded in the type editor
                        let Assemblaged { built_from } = asmblgd.get(ent).unwrap();
                        let hitbox = asmblgr.assemblages[built_from]
                            .iter()
                            .find(|x| x.name() == "Hitbox")
                            .unwrap()
                            .downcast_ref::<Hitbox>()
                            .unwrap();

                        // set the location to the combination of the two
                        ps.set_location(&phys, &(ground_collision_pos + hitbox.position));
                    }

                    if mouse_clicked_this_frame {
                        compium.mouselock_chosen_ent = false;
                    }
                }
            }
        }

        // if we don't have anything to place, but they've clicked,
        // they're probably trying to select something.
        if mouse_clicked_this_frame && !compium.mouselock_chosen_ent {
            let raycaster = Raycaster::point_from_camera(&ls.mouse_pos, &ls);
            let clicked_body_handle = ps
                .world
                .collider_world()
                .interferences_with_ray(&raycaster.ray, &raycaster.collision_group)
                .next()
                .unwrap()
                .0 //  it's (collider, rayhit), we want collider.body()
                .body();

            if let Ok(id) =
                serde_json::from_str(&ps.world.body(clicked_body_handle).unwrap().name())
            {
                compium.choose_ent(ents.entity(id), &ents, &mut outlines);
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
        use glutin::VirtualKeyCode::{LControl, S};
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
    type SystemData = (
        WriteExpect<'a, PhysState>,
        ReadStorage<'a, Phys>,
        ReadStorage<'a, EmitCollideEvent>,
        WriteStorage<'a, ScriptEvent>,
    );

    fn setup(&mut self, res: &mut specs::Resources) {
        Self::SystemData::setup(res);
        self.reader_id = Some(WriteStorage::<Phys>::fetch(&res).register_reader());
    }

    fn run(&mut self, (mut ps, physes, collides, mut script_events): Self::SystemData) {
        use nphysics3d::{
            math::{Force, ForceType},
            object::{Body, BodyHandle},
        };

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
            // perhaps replace that 0.5_f32 with a fraction of the actual velocity.
            let force = 2.0_f32.min(body.augmented_mass().linear * glm::length(lv) / timestep);
            let linear_force = -lv.normalize() * force;

            if force != 0.0 {
                body.apply_force(0, &Force::linear(linear_force), ForceType::Force, true);
            }
        }

        ps.world.step();

        for contact in ps.world.contact_events() {
            use ncollide3d::events::ContactEvent::*;

            if let Started(handle_one, handle_two) = contact {
                let ent_one = ps
                    .rbd_from_collider_handle(handle_one)
                    .and_then(|x| x.user_data())
                    .and_then(|x| x.downcast_ref::<specs::Entity>())
                    .map(|x| x.clone());
                let ent_two = ps
                    .rbd_from_collider_handle(handle_two)
                    .and_then(|x| x.user_data())
                    .and_then(|x| x.downcast_ref::<specs::Entity>())
                    .map(|x| x.clone());
                if let (Some(ent_one), Some(ent_two)) = (ent_one, ent_two) {
                    if let Some(EmitCollideEvent { script }) = collides.get(ent_one) {
                        script_events
                            .insert(ent_one, script.clone_with_payload(ent_two.id()))
                            .unwrap();
                    }
                    if let Some(EmitCollideEvent { script }) = collides.get(ent_two) {
                        script_events
                            .insert(ent_two, script.clone_with_payload(ent_one.id()))
                            .unwrap();
                    }
                }
            }
        }
    }
}

struct SpriteSheetAnimate;
impl<'a> System<'a> for SpriteSheetAnimate {
    type SystemData = (
        WriteStorage<'a, Appearance>,
        ReadStorage<'a, Animation>,
        ReadExpect<'a, LocalState>,
        ReadExpect<'a, SpritesheetDimensions>,
    );

    fn run(&mut self, (mut appearances, animations, local_state, sss): Self::SystemData) {
        for (app, ani) in (&mut appearances, &animations).join() {
            let frame_index =
                (local_state.elapsed_time * ani.fps).floor() % (ani.frame_count as f32);

            let x_size = app.size[0] * 64.0;
            app.uvs[0] = (x_size * frame_index) / sss.x;
            app.uvs[2] = (x_size * frame_index + x_size) / sss.x;
        }
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

    simple_logger::init_with_level(log::Level::Debug).unwrap();

    // -- Specs Resources:
    // windowing stuff
    let (mut glutin_state, window) = GlutinState::new(
        "stockRPG",
        glutin::dpi::LogicalSize {
            width: 1366.0,
            height: 768.0,
        },
    )
    .unwrap();
    let mut local_state = LocalState::from_glutin_window(&window.window());
    // Developer Tools stuff
    let mut dev_ui = DevUiState::new(&window.window());
    let compendium = Compendium::new();
    // rendering
    let (mut glium, spritesheet_size) = GliumState::new(window, &mut dev_ui.imgui);
    // Dyon
    let mut dyon_state = DyonState::new();
    let dyon_console = DyonConsole::default();
    // physics
    let physics_state = PhysState::new();
    // spritesheet texture indexes
    let image_bundle = ImageBundle::new();

    let mut world = World::new();
    #[rustfmt::skip]
    let mut dispatcher = DispatcherBuilder::new()
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
    world.register::<BoxOutline>();
    world.register::<Phys>();
    register!(
        AppearanceBuilder,
        MovementControls,
        EmitCollideEvent,
        ScriptingIds,
        Interactable,
        ScriptEvent,
        Explodeable,
        CameraFocus,
        Animation,
        Hitbox,
        Health,
    );

    assemblager.load_save(&mut world);
    local_state.find_camera_focus_and_zoom(&world);

    world.add_resource(spritesheet_size);
    world.add_resource(physics_state);
    world.add_resource(image_bundle);
    world.add_resource(dyon_console);
    world.add_resource(assemblager);
    world.add_resource(local_state);
    world.add_resource(compendium);

    while !world.read_resource::<LocalState>().quit {
        // input deals with thread-bound stuff so it's not a system
        glutin_state.input(&*glium.display.gl_window(), &world, &mut dev_ui);

        // your everyday ECS systems are run first
        dispatcher.dispatch(&mut world.res);

        // the systems can add things that scripts might want
        world.maintain();

        // the scripting system completely breaks ECS
        let specs_world_guard = CurrentGuard::new(&mut world);
        dyon_state.run();
        drop(specs_world_guard);

        // the scripts can add things lazily
        world.maintain();

        // next, the developer UI is generated based on all that.
        let ui = dev_ui.run(&world);

        // and the dev_ui is rendered right alongside the world.
        glium.render(&world, ui);
    }
}
