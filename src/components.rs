use crate::prelude::*;

use custom_component_macro_derive::*;
use custom_component_macro::*;
use serde::{Serialize, Deserialize};
use specs::{
    Component, VecStorage, DenseVecStorage, HashMapStorage, FlaggedStorage,
};
use specs_derive::Component;


//this component gives the compendium and save/load system a reference point for entity composition

#[derive(Default, Component, DevUiComponent, CopyToOtherEntity, AssemblageComponent, Serialize, Deserialize, Debug, Clone)]
#[storage(DenseVecStorage)]
pub struct Assemblaged {
    pub built_from: String,
}
impl DevUiRender for Assemblaged {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        ui.text(imgui::im_str!("Assembled From: "));
    }
}


//game mechanics components

#[derive(Default, Component, DevUiComponent, CopyToOtherEntity, AssemblageComponent, Serialize, Deserialize, Clone, Debug)]
pub struct Interactable {
    pub message: String,
}
impl DevUiRender for Interactable {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("Interactable"));

        let mut im_message = ImString::with_capacity(100);//self.message.len() + 1);
        im_message.push_str(&self.message);
        if ui.input_text_multiline(im_str!("< Message"), &mut im_message, [300.0, 45.0]).build() {
            self.message = im_message.to_str().to_owned();    
        }
        //dbg!(self.message.as_bytes());
    }
}


// rendering related components! 

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Appearance {
    pub uv_rect: [f32; 4],
}

#[derive(Default, Component, CopyToOtherEntity, AssemblageComponent, Serialize, Deserialize, Debug, Clone)]
pub struct AppearanceBuilder {
    pub image_name: String,
    pub uv_override: [f32; 4],
    pub built: bool,
}
//AnimationBuilder is weird, because they aren't actually used for anything, and are immediately
//turned into Animations when detected
impl DevUiComponent for AppearanceBuilder {
    fn ui_for_entity(&self, ui: &imgui::Ui, world: &specs::World, ent: &specs::Entity) {
        let mut appearances = world.write_storage::<Appearance>();
        let app = appearances.get_mut(*ent).unwrap();
        use imgui::*;

        ui.text(im_str!("Appearance"));

        for (index, coord) in app.uv_rect.iter().enumerate() {
            ui.label_text(
                im_str!("uv coordinate #{}", index),
                im_str!("{}", coord)
            );
        }
    }
}
impl DevUiRender for AppearanceBuilder {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, world: &specs::World) {
        use imgui::*;
        let image_bundle = world.read_resource::<ImageBundle>();

        ui.text(im_str!("Appearance"));

        let image_names = image_bundle.map
            .keys()
            .map(|x| ImString::new(x.clone()))
            .collect::<Vec<_>>();

        let image_im_str_names = image_names
            .iter()
            .map(ImStr::new)
            .collect::<Vec<_>>();

        let mut name_index = if let Some(index) = image_bundle.map.keys().position(|x| *x == self.image_name) {
            index
        } else {
            ui.text(im_str!("error, couldn't find the name of the image being used."));
            0
        } as i32;

        if ui.combo(
            im_str!("Image Names"),
            &mut name_index,
            &image_im_str_names,
            12,
        ) {
            self.image_name = image_im_str_names[name_index as usize].to_str().to_owned();
        }


        let size: &mut [f32; 2] = &mut [self.uv_override[2], self.uv_override[3]];
        ui.input_float(im_str!("uv coord 1"), &mut self.uv_override[0])
            .step(32.0)
            .build();
        ui.input_float(im_str!("uv coord 2"), &mut self.uv_override[1])
            .step(32.0)
            .build();
        ui.drag_float2(im_str!("uv size"), size).build();
        self.uv_override = [self.uv_override[0], self.uv_override[1], size[0], size[1]];
    }
}

#[derive(Default, Component, DevUiComponent, CopyToOtherEntity, AssemblageComponent, Serialize, Deserialize, Debug, Clone)]
#[storage(VecStorage)]
pub struct Animation {
    pub frame_count: i32,
    pub fps: f32,
}
impl DevUiRender for Animation {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("Animation"));

        ui.drag_float(im_str!("fps"), &mut self.fps)
            .power(0.01)
            .speed(0.01)
            .build();
        ui.input_int(im_str!("frame count"), &mut self.frame_count)
            .step(1)
            .build();
    }
}


//the next two are kinda flaggy components that only one entity should have.

#[derive(Default, Component, DevUiComponent, CopyToOtherEntity, AssemblageComponent, Serialize, Deserialize, Debug, Clone)]
#[storage(HashMapStorage)]
pub struct MovementControls {
    pub speed: f32,
}
impl DevUiRender for MovementControls {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;

        ui.text(im_str!("MovementControls"));

        ui.slider_float(
            im_str!("speed"),
            &mut self.speed,
            0.0,
            20.0,
        ).build();
    }
}


// some physics components!

#[derive(Component, AssemblageComponent, PartialEq, Serialize, Deserialize, Debug, Clone)]
#[storage(HashMapStorage)] //this exists for literally the end of one game loop; few will have it.
pub struct Hitbox {
    //sure, I could use a matrix, but let's try to make this
    //somewhat human readable when it's stored as JSON.
    //(also, you can't easily define a hitbox from a matrix using the API the physics engine exposes)
    pub position: glm::TVec3<f32>,
    pub rotation: glm::TVec3<f32>,
    pub scale: glm::TVec3<f32>,
    pub density: f32,
    pub physics_interaction: bool,
}
impl Default for Hitbox {
    fn default () -> Self {
        Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::vec3(0.0, 0.0, 0.0),
            scale: glm::vec3(0.5, 0.5, 0.5),
            density: 1.0,
            physics_interaction: true,
        }
    }
}
impl CopyToOtherEntity for Hitbox {
    fn copy_self_to(&self, world: &specs::World, ent: &specs::Entity) {
        let mut hitboxes = world.write_storage::<Self>();
        let physes = world.read_storage::<Phys>();
        let ps = world.read_resource::<PhysState>();
        let phys = physes.get(*ent).unwrap();

        //the only thing they get to keep is their location + rotation
        hitboxes.insert(*ent, Hitbox {
            position: *ps.location(phys).unwrap(),
            rotation: ps.euler_vec(phys).unwrap(),
            ..self.clone()
        }).unwrap();
    }
}
//hitbox is weird, because changing a hitbox that exists ingame as a phys is fundamentally
//different than changing a hitbox that exists in a save file just laying around.
impl DevUiComponent for Hitbox {
    fn ui_for_entity(&self, ui: &imgui::Ui, world: &specs::World, ent: &specs::Entity) {
        use nalgebra::geometry::{Isometry, Translation, UnitQuaternion};
        use ncollide3d::shape::Cuboid;
        let mut physes = world.write_storage::<Phys>();
        let phys = physes.get_mut(*ent).unwrap();
        let mut ps = world.write_resource::<PhysState>();

        if let Some(body) = ps.rigid_body(phys) {

            let position = body.position();
            let handle = body.handle();

            let mut hitbox = Hitbox {
                position: position.translation.vector,
                rotation: {
                    let eulers = position.rotation.euler_angles();
                    glm::vec3(eulers.0, eulers.1, eulers.2)
                },
                scale: *ps.collider(&phys).unwrap().shape().as_shape::<Cuboid<f32>>().unwrap().half_extents(),
                density: 1.0,
                physics_interaction: ps.do_physics_interact(&phys),
            };
            let old_hitbox = hitbox.clone();

            hitbox.dev_ui_render(&ui, &world);

            //if the dev ui changed the component,
            if hitbox != old_hitbox {

                //for scale and physics_interaction, we need to rebuild the entire Phys, but if
                //position's all they've changed, we can just move them.
                if hitbox.scale == old_hitbox.scale && hitbox.physics_interaction == old_hitbox.physics_interaction {
                    if let Some(body) = ps.rigid_body_mut(phys) {
                        body.set_position(Isometry::from_parts(
                            Translation::from(hitbox.position),
                            UnitQuaternion::from_euler_angles(
                                hitbox.rotation.x,
                                hitbox.rotation.y,
                                hitbox.rotation.z
                            ),
                        ));
                    }
                }

                else {
                    //we completely obliterate that peon
                    ps.world.remove_bodies(&[handle]);

                    //aaand make a new one
                    let phys_comp = ps.phys_from_hitbox(&mut hitbox);
                    //add the entity as some user data so that when we find it via raycasting
                    //we can detect it for what it is.
                    ps.name_as_ent(&phys_comp, Box::new(*ent));
                    //add the new physics body to the world.
                    physes.insert(*ent, phys_comp).unwrap();
                }
            }
        }
    }
}
impl DevUiRender for Hitbox {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;
        use std::convert::TryInto;

        ui.text(im_str!("Hitbox"));

        for rad in self.rotation.iter_mut() { *rad = rad.to_degrees(); }

        ui.drag_float3(im_str!("position"), self.position.data.as_mut_slice().try_into().unwrap())
            .speed(0.1)
            .build();
        ui.drag_float3(im_str!("rotation"), self.rotation.data.as_mut_slice().try_into().unwrap())
            .min(-360.0)
            .max( 360.0)
            .build();
        ui.drag_float3(im_str!("scale"), self.scale.data.as_mut_slice().try_into().unwrap())
            .min(0.1)
            .max(10.0)
            .speed(0.0001)
            .build();

        for deg in self.rotation.iter_mut() { *deg = deg.to_radians(); }

        ui.checkbox(im_str!("< physics interaction"), &mut self.physics_interaction);
    }
}
#[derive(Debug, Clone)]
pub struct Phys {
    pub body: nphysics3d::object::BodyHandle,
}
impl Component for Phys {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}
