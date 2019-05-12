use super::prelude::*;

// some physics components!

#[derive(
    Component,
    DevUiComponent,
    CopyToOtherEntity,
    AssemblageComponent,
    Serialize,
    Deserialize,
    Clone,
    Debug,
)]
#[storage(VecStorage)]
pub struct ApplyForce {
    pub vec: glm::TVec3<f32>,
    pub time_elapsed: f32,
    pub duration: f32,
    pub decay: f32,
}
impl Default for ApplyForce {
    fn default() -> Self {
        Self {
            vec: glm::vec3(0.0, 0.0, 0.0),
            time_elapsed: 0.0,
            duration: 0.15,
            decay: 1.0,
        }
    }
}
#[allow(dead_code)]
impl ApplyForce {
    pub fn random_2d_vec() -> glm::TVec3<f32> {
        use nalgebra::Vector3;
        use std::f32;

        let angle = OsRng::new().unwrap().gen_range(0.0, f32::consts::PI * 2.0);
        Vector3::new(angle.cos(), angle.sin(), 0.0)
    }
    pub fn random_2d_force_with_magnitude(magnitude: f32) -> Self {
        Self {
            vec: Self::random_2d_vec() * magnitude,
            ..Self::default()
        }
    }
}
impl DevUiRender for ApplyForce {
    fn dev_ui_render(&mut self, ui: &imgui::Ui, _world: &specs::World) {
        use imgui::*;
        use std::convert::TryInto;

        ui.drag_float3(
            im_str!(""),
            self.vec.data.as_mut_slice().try_into().unwrap(),
        )
        .speed(0.1)
        .build();

        ui.input_float(im_str!("duration"), &mut self.duration)
            .step(0.01)
            .build();

        ui.input_float(im_str!("decay"), &mut self.decay)
            .step(0.01)
            .build();
    }
}

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
    fn default() -> Self {
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
        hitboxes
            .insert(
                *ent,
                Hitbox {
                    position: *ps.location(phys).unwrap(),
                    rotation: ps.euler_vec(phys).unwrap(),
                    ..self.clone()
                },
            )
            .unwrap();
    }
}
//hitbox is weird, because changing a hitbox that exists ingame as a phys is fundamentally
//different than changing a hitbox that exists in a save file just laying around.
impl DevUiComponent for Hitbox {
    fn ui_for_entity(&self, ui: &imgui::Ui, world: &specs::World, ent: &specs::Entity) {
        let mut physes = world.write_storage::<Phys>();
        let phys = physes.get_mut(*ent).unwrap();
        let mut ps = world.write_resource::<PhysState>();

        if let Some(body) = ps.rigid_body(phys) {
            let handle = body.handle();

            let mut hitbox = ps.hitbox_from_phys(&phys);
            let old_hitbox = hitbox.clone();

            hitbox.dev_ui_render(&ui, &world);

            //if the dev ui changed the component,
            if hitbox != old_hitbox {
                //for scale and physics_interaction, we need to rebuild the entire Phys, but if
                //position's all they've changed, we can just move them.
                if hitbox.scale == old_hitbox.scale
                    && hitbox.physics_interaction == old_hitbox.physics_interaction
                {
                    ps.set_position(&phys, &hitbox.position, &hitbox.rotation);
                } else {
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

        for rad in self.rotation.iter_mut() {
            *rad = rad.to_degrees();
        }

        ui.drag_float3(
            im_str!("position"),
            self.position.data.as_mut_slice().try_into().unwrap(),
        )
        .speed(0.1)
        .build();

        ui.drag_float3(
            im_str!("rotation"),
            self.rotation.data.as_mut_slice().try_into().unwrap(),
        )
        .min(-360.0)
        .max(360.0)
        .build();

        ui.drag_float3(
            im_str!("scale"),
            self.scale.data.as_mut_slice().try_into().unwrap(),
        )
        .min(0.1)
        .max(10.0)
        .speed(0.0001)
        .build();

        for deg in self.rotation.iter_mut() {
            *deg = deg.to_radians();
        }

        ui.checkbox(
            im_str!("< physics interaction"),
            &mut self.physics_interaction,
        );
    }
}
#[derive(Debug, Clone)]
pub struct Phys {
    pub body: nphysics3d::object::BodyHandle,
}
impl Component for Phys {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}
