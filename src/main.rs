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
        use specs::Join;

        //for every AppearanceBuilder in the world, delete it, but then do
        for (ent, mut appear_builder) in (&*entities, &mut appear_builders).join() {
            if !appear_builder.built {
                appear_builder.built = true;
                if let Some(base_u32) = image_bundle.map.get(&appear_builder.image_name) {
                    let base = {
                        //yes I coulda used a vec but I really want an array okay
                        let mut iter = base_u32.iter().map(|&x| x as f32);
                        [
                            iter.next().unwrap(),
                            iter.next().unwrap(),
                            iter.next().unwrap(),
                            iter.next().unwrap(),
                        ]
                    };
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
                info!("{}", force.duration);

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
        ReadExpect<'a, LocalState>,
        ReadExpect<'a, PhysState>,
        ReadStorage<'a, Phys>,
        ReadStorage<'a, Interactable>,
        ReadStorage<'a, MovementControls>,
    );

    fn run(
        &mut self,
        (local_state, ps, physes, interactables, movement_controls): Self::SystemData,
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

            let closest_interactable: Option<&Interactable> = (&physes, &interactables)
                .join()
                //turn each (phys, interactable) into (distance, interactable)
                .map(|(phys, interactable)| {
                    let interactable_pos = ps.location(phys).unwrap().xy();
                    (glm::distance2(&player_pos, &interactable_pos), interactable)
                })
                //find the tuple with the smallest distance from the player
                .fold((GRABBING_RANGE, None), |acc, (distance, interactable)| {
                    if distance < acc.0 {
                        //acc.0 is the distance, of course.
                        //if this tuple's distance is smaller than acc's,
                        //return it from the closure so it's acc for the next iterations.
                        (distance, Some(interactable))
                    } else {
                        //if this tuple's distance isn't smaller,
                        //return acc so it stays the same.
                        acc
                    }
                })
                //turn (distance, interactable) into interactable
                .1;

            //check if the closest interactable to player is close enough
            if let Some(Interactable { message }) = closest_interactable {
                println!("{}", &message);
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

//this boi needs world access because he'll have to access storages dynamically
struct DevUiUpdate {
    dev_ui: DevUiState,
}
impl DevUiUpdate {
    fn new() -> Self {
        Self {
            dev_ui: DevUiState::new(),
        }
    }

    fn run(&mut self, world: &specs::World) {
        use imgui::*;
        use specs::Join;

        //resources
        let mut compium = world.write_resource::<Compendium>();
        let mut asmblgr = world.write_resource::<Assemblager>();
        let lu = world.read_resource::<LazyUpdate>();
        let ents = world.entities();
        //storages (still technically resources but you know)
        let mut assemblaged = world.write_storage::<Assemblaged>();
        let camera_focuses = world.read_storage::<CameraFocus>();
        let editing_assemblage = compium.editing_assemblage.clone();
        //extra state stuff
        let mut open_type_from_entity_modal = false;

        if let Some(CameraFocus { background_color }) = camera_focuses.join().next() {
            self.dev_ui.clear_color = *background_color;
        }

        self.dev_ui.update(|ui| {
            ui.show_metrics_window(&mut true);

            ui.with_style_var(StyleVar::WindowRounding(0.0), || {
                ui.window(im_str!("The Compendium"))
                    .size((375.0, 550.0), ImGuiCond::FirstUseEver)
                    .position((25.0, 25.0), ImGuiCond::FirstUseEver)
                    .build(|| {
                        ui.separator();

                        ui.input_text(im_str!("< Entity Query"), &mut compium.entity_query)
                            .build();

                        ui.separator();

                        if ui.button(im_str!("New Type"), [85.0, 20.0]) {
                            ui.open_popup(im_str!("Name Type"));
                        }
                        ui.popup_modal(im_str!("Name Type")).build(|| {
                            ui.text("What would you like to name the new type?");
                            ui.input_text(im_str!("< Name"), &mut compium.wip_type_name)
                                .build();

                            if ui.button(im_str!("That's it!"), (0.0, 0.0)) {
                                asmblgr
                                    .assemblages
                                    .insert(compium.wip_type_name.to_str().to_string(), Vec::new());
                                ui.close_current_popup();
                            }
                        });

                        ui.separator();

                        for (assemblage_key, _) in &asmblgr.assemblages {
                            if ui.selectable(
                                im_str!("{}", assemblage_key),
                                match &compium.place_assemblage {
                                    Some(chosen) => {
                                        chosen.to_string() == assemblage_key.to_string()
                                    }
                                    _ => false,
                                },
                                ImGuiSelectableFlags::empty(),
                                ImVec2::new(0.0, 0.0),
                            ) {
                                compium.place_assemblage = Some(assemblage_key.to_string());
                                compium.place_me_entity =
                                    Some(asmblgr.build(&assemblage_key, &lu, &ents));
                            }

                            if ui.is_item_hovered()
                                && ui.imgui().is_mouse_clicked(ImMouseButton::Right)
                            {
                                compium.editing_assemblage = match compium.editing_assemblage {
                                    Some(_) => None,
                                    None => Some(assemblage_key.to_string()),
                                }
                            }
                        }

                        ui.separator();

                        ui.text(im_str!("This...is...imgui-rs!"));
                        let mouse_pos = ui.imgui().mouse_pos();
                        ui.text(im_str!(
                            "Mouse Position: ({:.1},{:.1})",
                            mouse_pos.0,
                            mouse_pos.1
                        ));
                    });
            });

            if let Some(chose_ent) = compium.chosen_entity {
                if let Some(Assemblaged { built_from }) = assemblaged.get(chose_ent) {
                    ui.window(im_str!("{}", built_from))
                        .position((125.0, 300.0), ImGuiCond::FirstUseEver)
                        .size((345.0, 165.0), ImGuiCond::FirstUseEver)
                        .menu_bar(true)
                        .build(|| {
                            if ui.button(im_str!("Remove Entity"), [120.0, 20.0]) {
                                lu.exec_mut(move |world| {
                                    world.delete_entity(chose_ent).unwrap();
                                });
                                compium.chosen_entity = None;
                            }

                            ui.menu_bar(|| {
                                ui.menu(im_str!("Type Interactions")).build(|| {
                                    if ui.menu_item(im_str!("New type from this entity")).build() {
                                        open_type_from_entity_modal = true;
                                    }
                                });
                            });

                            //built_from is the key for the assemblage this entity was built from.

                            ui.separator();

                            for asmblg in asmblgr.assemblages.get_mut(built_from) {
                                for comp in asmblg.iter() {
                                    //now to get the actual storage, you'll need to pass in the
                                    //world too, as well as the applicable entity, because there's
                                    //no way we can use the type of the component in question
                                    //outside of a method on that component. but that'll just be
                                    //changing the default macro.
                                    //this is really dumb, but basically instead of editing the
                                    //actual components we're iterating over, this edits the
                                    //component of the entity provided that is the same type as
                                    //this specific component. questionable design decision I know
                                    comp.ui_for_entity(&ui, &world, &chose_ent);
                                    ui.separator();
                                }
                            }
                        });
                }
            }

            //this opens the little modal window for creating new a type starting with an
            //already existing entity.
            if open_type_from_entity_modal {
                ui.open_popup(im_str!("New Type From Entity"));
            }
            ui.popup_modal(im_str!("New Type From Entity")).build(|| {
                ui.text("What would you like to name the new type?");

                ui.input_text(im_str!("< Name"), &mut compium.wip_type_name)
                    .build();

                if ui.button(im_str!("That's it!"), (0.0, 0.0)) {
                    //get the data about the entity that we need
                    let chose_ent = compium.chosen_entity.unwrap();
                    let built_from = assemblaged.get(chose_ent).unwrap().built_from.clone();

                    //make the components for the new type
                    let cloned_components = asmblgr.assemblages[&built_from]
                        .iter()
                        .map(|c| c.boxed_clone())
                        .collect::<Vec<_>>();
                    //ease of use copy of the string since it's used to make the new type and add
                    //the entity to the new type.
                    let assemblage_name_string = compium.wip_type_name.to_str().to_string();

                    //insert the new type that was just made
                    asmblgr
                        .assemblages
                        .insert(assemblage_name_string.clone(), cloned_components);
                    //move the entity to the new type
                    assemblaged
                        .insert(
                            chose_ent,
                            Assemblaged {
                                built_from: assemblage_name_string,
                            },
                        )
                        .unwrap();

                    //since we've gotten the information we needed and made the new type...
                    ui.close_current_popup();
                }
            });

            //THIS one on the other hand, edits the actual components stored
            if let Some(assemblage_key) = &editing_assemblage {
                ui.window(im_str!("Type Editor: {}", assemblage_key))
                    .position((25.0, 100.0), ImGuiCond::FirstUseEver)
                    .size((445.0, 345.0), ImGuiCond::FirstUseEver)
                    .build(|| {
                        if ui.button(im_str!("Push Changes"), [120.0, 20.0]) {
                            for (Assemblaged { built_from }, ent) in (&assemblaged, &ents).join() {
                                info!("{}", built_from);
                                if built_from == assemblage_key {
                                    for comp in asmblgr.assemblages[assemblage_key].iter() {
                                        comp.copy_self_to(&world, &ent);
                                    }
                                }
                            }
                        } else if ui.is_item_hovered() {
                            ui.tooltip_text(im_str!(
                                "This will update all instances
of this type with these stats.
Later each instance should just
store how different it is from the
original."
                            ));
                        }

                        ui.same_line(120.0 + 35.0);

                        if ui.button(im_str!("Add Component"), [120.0, 20.0]) {
                            ui.open_popup(im_str!("Select New Component"));
                        }

                        ui.text(im_str!("NOTE: Changes will be pushed on save."));
                        ui.text(im_str!("If separate functionality is desired,"));
                        ui.text(im_str!("a new type should be made."));

                        ui.popup_modal(im_str!("Select New Component")).build(|| {
                            ui.text("Which component would you like to add?");

                            //I have to have this weird construct to avoid copying the entire
                            //names_list just to avoid borrow errors. SIGH.
                            let add_me: Option<Box<custom_component_macro::AssemblageComponent>> = {
                                let comp_names_list = asmblgr
                                    .components
                                    .keys()
                                    .map(ImStr::new)
                                    .collect::<Vec<_>>();

                                ui.combo(
                                    im_str!("< Component To Add"),
                                    &mut compium.component_to_add_index,
                                    &comp_names_list,
                                    7,
                                );

                                if ui.button(im_str!("This one!"), [120.0, 20.0]) {
                                    let index = compium.component_to_add_index as usize;
                                    let component_name = comp_names_list[index];
                                    Some(asmblgr.components[component_name].boxed_clone())
                                } else {
                                    None
                                }
                            };

                            if let Some(component) = add_me {
                                let assemblage =
                                    asmblgr.assemblages.get_mut(assemblage_key).unwrap();
                                assemblage.push(component);
                                ui.close_current_popup();
                            }

                            ui.same_line(120.0 + 15.0);

                            if ui.button(im_str!("Nevermind."), [120.0, 20.0]) {
                                ui.close_current_popup();
                            }
                        });

                        ui.separator();
                        for comp in asmblgr
                            .assemblages
                            .get_mut(assemblage_key)
                            .unwrap()
                            .iter_mut()
                        {
                            comp.dev_ui_render(&ui, &world);
                            ui.separator();
                        }
                    });
            }
        });
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
    );

    fn run(&mut self, (physes, local_state, entities, mut ps, mut compium): Self::SystemData) {
        let mouse_clicked_this_frame = local_state.last_input.mouse_state.unwrap_or(false);

        if let Some(ent) = compium.place_me_entity {
            if let Some(phys) = physes.get(ent) {
                //we could get local_state.mouse_pos, but that's simply the last known mouse_pos.
                //we want the last_input one, since that'll tell us whether or not they moved the mouse
                //this frame; that'll let us only move the thing when we really need to.
                if let Some(mouse_pos) = local_state.last_input.mouse_pos {
                    let raycaster = Raycaster::point_from_camera(&mouse_pos, &local_state);
                    let ground_collision_pos = raycaster.cast_to_ground_pos(&ps).unwrap();
                    ps.set_location(&phys, &ground_collision_pos);
                }

                if mouse_clicked_this_frame {
                    let raycaster =
                        Raycaster::point_from_camera(&local_state.mouse_pos, &local_state);
                    let ground_collision_pos = raycaster.cast_to_ground_pos(&ps).unwrap();
                    ps.set_location(&phys, &ground_collision_pos);
                    println!("click!");
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
    type SystemData = (
        WriteExpect<'a, LocalState>,
        ReadExpect<'a, PhysState>,
        ReadStorage<'a, Phys>,
        ReadStorage<'a, MovementControls>,
    );

    fn run(&mut self, (mut local_state, ps, physes, movement_controls): Self::SystemData) {
        use specs::Join;

        let inputs = UserInput::poll_events_loop(&mut self.winit_state);
        /* if inputs.new_frame_size.is_some() {
            debug!("Window changed size, changing HalState...");
            hal_state.resize_swapchain();
        }*/
        local_state.update_from_input(inputs);

        for (phys, _mov) in (&physes, &movement_controls).join() {
            let dur = local_state.last_frame_duration;
            local_state.camera.lerp_towards(
                ps.rigid_body(phys).unwrap().position().translation.vector
                    + glm::vec3(-0.5, 0.0, 0.0),
                dur,
            );
        }
    }
}

fn main() {
    simple_logger::init().unwrap();

    //-- Specs Resources:
    //Developer Tools stuff
    let mut dev_ui = DevUiUpdate::new();
    let compendium = Compendium::new();
    //spritesheet texture indexes
    let image_bundle = ImageBundle::new();
    //windowing stuff
    let winit_state = WinitState::default();
    let local_state = LocalState::from_winit_state(&winit_state);
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
        ($name:ty) => {
            world.register::<$name>();
            let default: $name = Default::default();
            assemblager.register_component(stringify!($name).to_string(), default);
        };
    }

    world.register::<Assemblaged>();
    world.register::<Appearance>();
    world.register::<Phys>();
    register!(AppearanceBuilder);
    register!(MovementControls);
    register!(Interactable);
    register!(Explodeable);
    register!(CameraFocus);
    register!(Animation);
    register!(Hitbox);

    assemblager.load_save(&mut world);

    world.add_resource(compendium);
    world.add_resource(image_bundle);
    world.add_resource(hal_state);
    world.add_resource(local_state);
    world.add_resource(physics_state);
    world.add_resource(assemblager);

    while !world.read_resource::<LocalState>().quit {
        dispatcher.dispatch(&mut world.res);
        dev_ui.run(&world);
        world.maintain();
    }
}
