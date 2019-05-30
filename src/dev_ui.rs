// general
use crate::prelude::*;
use specs::LazyUpdate;

// imgui
use imgui::{FontGlyphRange, ImFontConfig, ImGui, Ui};
use imgui_winit_support as imwinit;

// this boi needs world access because he'll have to access storages dynamically
pub struct DevUiUpdate {
    pub dev_ui: DevUiState,
}

use imgui::*;
impl DevUiUpdate {
    pub fn new(window: &winit::Window) -> Self {
        Self {
            dev_ui: DevUiState::new(window),
        }
    }

    pub fn run(&mut self, world: &specs::World) -> Ui {

        // extra state stuff
        let mut open_type_from_entity_modal = false;

        // gotta make sure the compendium is dropped
        let (is_chosen_entity, is_type_to_edit) = {
            let compium = world.read_resource::<Compendium>();
            let ents = world.entities();
            (
                compium
                    .chosen_entity
                    .and_then(|x| ents.is_alive(x).as_option())
                    .is_some(),
                compium.editing_assemblage.is_some(),
            )
        };

        // imgui needs this to do its math.
        let delta_s = {
            let ls = world.read_resource::<LocalState>();
            ls.last_frame_duration.clone()
        };

        let ui = self.dev_ui.update(delta_s.into());

        // render the right-click-a-compendium-type thing
        // this actually has to get rendered before the compendium,
        // or is_type_to_edit could become invalid.
        if is_type_to_edit {
            ui.window(im_str!("Type Editor"))
                .position((1366.0 - 445.0, 0.0), ImGuiCond::FirstUseEver)
                .size((445.0, 345.0), ImGuiCond::FirstUseEver)
                .menu_bar(true)
                .build(|| Self::render_type_editor(&ui, &world));
        }

        // render the you-clicked-an-entity thing
        if is_chosen_entity {
            ui.window(im_str!("Entity Editor"))
                .position((1366.0 - 445.0, 20.0), ImGuiCond::FirstUseEver)
                .size((445.0, 345.0), ImGuiCond::FirstUseEver)
                .menu_bar(true)
                .build(|| {
                    open_type_from_entity_modal = Self::render_entity_editor(&ui, &world);
                });
        }

        // render the fancy compendium thing
        ui.with_style_var(StyleVar::WindowRounding(0.0), || {
            ui.window(im_str!("The Compendium"))
                .position((0.0, 0.0), ImGuiCond::FirstUseEver)
                .size((250.0, 500.0), ImGuiCond::FirstUseEver)
                .build(|| {
                    Self::render_compendium(&ui, &world);
                });
        });

        // show the little window with the FPS in it
        ui.show_metrics_window(&mut true);

        // for the scripteeronators!!!
        ui.window(im_str!("Dyon Console"))
            .position((0.0, 768.0 - 270.0), ImGuiCond::FirstUseEver)
            .size((475.0, 270.0), ImGuiCond::FirstUseEver)
            .build(|| {
                let dyon_console = &world.read_resource::<DyonConsole>().0;
                for message in dyon_console.split('\n') {
                    ui.text(im_str!("{}", message));
                }
            });

        // this opens the little modal window for creating new a type starting with an
        // already existing entity.
        if open_type_from_entity_modal {
            ui.open_popup(im_str!("New Type From Entity"));
        }
        ui.popup_modal(im_str!("New Type From Entity")).build(|| {
            Self::render_add_type_popup(&ui, &world);
        });

        ui
    }

    #[inline]
    fn render_type_editor(ui: &Ui, world: &specs::World) {
        use specs::Join;

        // resources
        let mut compium = world.write_resource::<Compendium>();
        let mut asmblgr = world.write_resource::<Assemblager>();
        let ents = world.entities();
        // storages (still technically resources but you know)
        let assemblaged = world.read_storage::<Assemblaged>();

        let assemblage_key = &compium.editing_assemblage.clone().unwrap();

        // https:// github.com/ocornut/imgui/issues/331
        let mut component_remove_modal = false;
        let mut component_add_modal = false;
        let mut should_push_changes = false;

        ui.menu_bar(|| {
            ui.menu(im_str!("Components")).build(|| {
                if ui.menu_item(im_str!("New Component")).build() {
                    component_add_modal = true;
                }
                if ui.menu_item(im_str!("Remove Component")).build() {
                    component_remove_modal = true;
                }
            });
        });

        if component_add_modal {
            ui.open_popup(im_str!("Add Component"));
        }

        if component_remove_modal {
            ui.open_popup(im_str!("Remove Component"));
        }

        ui.popup_modal(im_str!("Add Component")).build(|| {
            ui.text("Which component would you like to add?");

            // I have to have this weird construct to avoid copying the entire
            // names_list just to avoid borrow errors. Safety! :D
            let add_me: Option<Box<custom_component_macro::AssemblageComponent>> = {
                let existing_comps = asmblgr.assemblages[assemblage_key]
                    .iter()
                    .map(|x| x.name())
                    .collect::<Vec<_>>();

                let comp_names = asmblgr
                    .components
                    .keys()
                    .filter(|x| !existing_comps.contains(&x.to_str()))
                    .map(ImStr::new)
                    .collect::<Vec<_>>();

                ui.combo(
                    im_str!("< Component To Add"),
                    &mut compium.component_to_add_index,
                    &comp_names,
                    20,
                );

                if ui.button(im_str!("This one!"), [120.0, 20.0]) {
                    let index = compium.component_to_add_index as usize;
                    let component_name = comp_names[index];
                    Some(asmblgr.components[component_name].boxed_clone())
                } else {
                    None
                }
            };

            if let Some(component) = add_me {
                let assemblage = asmblgr.assemblages.get_mut(assemblage_key).unwrap();
                assemblage.push(component);
                should_push_changes = true;
                ui.close_current_popup();
            }

            ui.same_line(120.0 + 15.0);

            if ui.button(im_str!("Nevermind."), [120.0, 20.0]) {
                ui.close_current_popup();
            }
        });

        ui.popup_modal(im_str!("Remove Component")).build(|| {
            ui.text(im_str!("Which component would you like to remove?"));
            let comp_names = asmblgr.assemblages[assemblage_key]
                .iter()
                .map(|x| x.name())
                .collect::<Vec<_>>();
            let comp_strings = comp_names
                .iter()
                .map(|x| ImString::new(x.to_owned()))
                .collect::<Vec<_>>();
            let comp_strs = comp_strings.iter().map(ImStr::new).collect::<Vec<_>>();

            ui.combo(
                im_str!("< Component To Remove"),
                &mut compium.component_to_add_index,
                &comp_strs,
                5,
            );

            if ui.button(im_str!("This one!"), [120.0, 20.0]) {
                asmblgr
                    .assemblages
                    .get_mut(assemblage_key)
                    .unwrap()
                    .remove(compium.component_to_add_index as usize);
                should_push_changes = true;
                ui.close_current_popup();
            }

            ui.same_line(120.0 + 15.0);

            if ui.button(im_str!("Nevermind."), [120.0, 20.0]) {
                ui.close_current_popup();
            }
        });
        // end of modals

        if ui.button(im_str!("Push Changes"), [140.0, 20.0]) || should_push_changes {
            for (Assemblaged { built_from }, ent) in (&assemblaged, &ents).join() {
                if built_from == assemblage_key {
                    for comp in asmblgr.assemblages[assemblage_key].iter() {
                        comp.copy_self_to(&world, &ent);
                    }
                }
            }
        } else if ui.is_item_hovered() {
            ui.tooltip_text(im_str!(
                "This will update all instances \
                 of this type with these stats. \
                 Later each instance should just \
                 store how different it is from the \
                 original."
            ));
        }

        ui.text(im_str!("NOTE: Changes will be pushed on save."));
        ui.text(im_str!("If separate functionality is desired,"));
        ui.text(im_str!("a new type should be made."));

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
    }

    #[inline]
    fn render_add_type_popup(ui: &Ui, world: &specs::World) {
        let mut compium = world.write_resource::<Compendium>();
        let mut asmblgr = world.write_resource::<Assemblager>();
        // storages (still technically resources but you know)
        let mut assemblaged = world.write_storage::<Assemblaged>();

        ui.text("What would you like to name the new type?");

        ui.input_text(im_str!("< Name"), &mut compium.wip_type_name)
            .build();

        if ui.button(im_str!("That's it!"), (0.0, 0.0)) {
            // get the data about the entity that we need
            let chose_ent = compium.chosen_entity.unwrap();
            let built_from = assemblaged.get(chose_ent).unwrap().built_from.clone();

            // make the components for the new type
            let cloned_components = asmblgr.assemblages[&built_from]
                .iter()
                .map(|c| c.boxed_clone())
                .collect::<Vec<_>>();
            // ease of use copy of the string since it's used to make the new type and add
            // the entity to the new type.
            let assemblage_name_string = compium.wip_type_name.to_str().to_string();

            // insert the new type that was just made
            asmblgr
                .assemblages
                .insert(assemblage_name_string.clone(), cloned_components);
            // move the entity to the new type
            assemblaged
                .insert(
                    chose_ent,
                    Assemblaged {
                        built_from: assemblage_name_string,
                    },
                )
                .unwrap();

            // since we've gotten the information we needed and made the new type...
            ui.close_current_popup();
        }
    }

    #[inline]
    fn render_entity_editor(ui: &Ui, world: &specs::World) -> bool {
        let mut open_type_from_entity_modal = false; // this is returned

        // resources
        let lu = world.read_resource::<LazyUpdate>();
        let mut compium = world.write_resource::<Compendium>();
        let mut asmblgr = world.write_resource::<Assemblager>();
        // storages (still technically resources but you know)
        let assemblaged = world.write_storage::<Assemblaged>();

        // this function wouldn't have been called if this could fail
        let chose_ent = compium.chosen_entity.unwrap();
        let Assemblaged { built_from } = assemblaged
            .get(chose_ent)
            .expect("Somehow whatever you clicked doesn't have an assemblaged component.");

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

        // built_from is the key for the assemblage this entity was built from.

        ui.separator();

        for asmblg in asmblgr.assemblages.get_mut(built_from) {
            for comp in asmblg.iter() {
                // now to get the actual storage, you'll need to pass in the
                // world too, as well as the applicable entity, because there's
                // no way we can use the type of the component in question
                // outside of a method on that component. but that'll just be
                // changing the default macro.
                // this is really dumb, but basically instead of editing the
                // actual components we're iterating over, this edits the
                // component of the entity provided that is the same type as
                // this specific component. questionable design decision I know
                comp.ui_for_entity(&ui, &world, &chose_ent);
                ui.separator();
            }
        }

        open_type_from_entity_modal
    }

    #[inline]
    fn render_compendium(ui: &Ui, world: &specs::World) {
        // resources
        let mut compium = world.write_resource::<Compendium>();
        let mut asmblgr = world.write_resource::<Assemblager>();
        let lu = world.read_resource::<LazyUpdate>();
        let ents = world.entities();
        // storages (still technically resources but you know)

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
                    Some(chosen) => chosen.to_string() == assemblage_key.to_string(),
                    _ => false,
                },
                ImGuiSelectableFlags::empty(),
                ImVec2::new(0.0, 0.0),
            ) {
                // delete whatever they were about to place before if that's a thing
                if let Some(old_entity) = compium.place_me_entity {
                    ents.delete(old_entity).unwrap();
                }

                // okay now make new things
                compium.place_assemblage = Some(assemblage_key.to_string());
                compium.place_me_entity = Some(asmblgr.build(&assemblage_key, &lu, &ents));
            }

            if ui.is_item_hovered() && ui.imgui().is_mouse_clicked(ImMouseButton::Right) {
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
    }
}

pub struct DevUiState {
    frame_size: imgui::FrameSize,
    pub imgui: ImGui,
}

impl DevUiState {
    pub fn new(window: &winit::Window) -> Self {
        let hidpi_factor = window.get_hidpi_factor().round();

        let mut imgui = ImGui::init();
        imgui.set_ini_filename(None);

        let font_size = (8.0 * hidpi_factor) as f32;

        imgui.fonts().add_font_with_config(
            include_bytes!("./font/SDS_8x8.ttf"),
            ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
            &FontGlyphRange::default(),
        );

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

        imwinit::configure_keys(&mut imgui);

        Self {
            imgui,
            frame_size: imwinit::get_frame_size(window, hidpi_factor).unwrap(),
        }
    }

    pub fn other_input_processing(&mut self, window: &winit::Window) {
        imwinit::update_mouse_cursor(&self.imgui, window);
        self.frame_size =
            imwinit::get_frame_size(window, window.get_hidpi_factor().round()).unwrap();
    }

    pub fn process_event(&mut self, event: &winit::Event, dpi_factor: f64) {
        let imgui = &mut self.imgui;

        imwinit::handle_event(imgui, &event, dpi_factor, dpi_factor);
    }

    pub fn update(&mut self, delta_s: f32) -> Ui {
        self.imgui.frame(self.frame_size, delta_s)
    }
}
