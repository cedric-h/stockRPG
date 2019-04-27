use crate::prelude::*;
use na::Vector3;
use nalgebra as na;
use ncollide3d::{
    query::RayIntersection,
    shape::{Cuboid, ShapeHandle},
    world::CollisionGroups,
};
use nphysics3d::{
    object::{Body, Collider, ColliderDesc, RigidBody, RigidBodyDesc},
    world::World,
};
const GROUND_SIZE: f32 = 50.0;

pub struct PhysState {
    pub world: World<f32>,
    pub can_collide_group: CollisionGroups,
    pub disabled_group: CollisionGroups,
    pub raycast_group: CollisionGroups,
}

#[allow(dead_code)]
impl PhysState {
    pub fn new() -> Self {
        let can_collide_group = CollisionGroups::new()
            .with_membership(&[0, 2])
            .with_whitelist(&[0, 2])
            .with_blacklist(&[1]);
        let disabled_group = CollisionGroups::new()
            .with_membership(&[1, 2])
            .with_whitelist(&[0, 2])
            .with_blacklist(&[0, 1]);
        let raycast_group = CollisionGroups::new()
            .with_membership(&[2])
            .with_whitelist(&[0, 1, 2])
            .with_blacklist(&[]);

        /*
        assert!(raycast_group.can_interact_with_groups(&disabled_group));
        assert!(raycast_group.can_interact_with_groups(&can_collide_group));
        assert!(!disabled_group.can_interact_with_groups(&can_collide_group));
        assert!(!disabled_group.can_interact_with_groups(&disabled_group));
        assert!(can_collide_group.can_interact_with_groups(&can_collide_group));*/

        let mut world = World::new();
        let ground = ShapeHandle::new(Cuboid::new(Vector3::repeat(GROUND_SIZE)));
        ColliderDesc::new(ground)
            .collision_groups(can_collide_group)
            .translation(Vector3::z() * -GROUND_SIZE)
            .build(&mut world);

        PhysState {
            world,
            can_collide_group,
            disabled_group,
            raycast_group,
        }
    }

    pub fn phys_from_hitbox(&mut self, hitbox: &mut Hitbox) -> Phys {
        //the scale acts as the hitbox's dimensions,
        let shape_handle = ShapeHandle::new(Cuboid::new(hitbox.scale));
        let collider_desc = ColliderDesc::new(shape_handle)
            //.translation(Vector3::z() * hitbox.scale.z/2.0)
            .density(hitbox.density)
            .collision_groups(if hitbox.physics_interaction {
                self.can_collide_group
            } else {
                self.disabled_group
            });

        let mut body_desc = RigidBodyDesc::new();
        body_desc.set_rotations_kinematic(Vector3::new(true, true, true));

        let mut body = body_desc.collider(&collider_desc).build(&mut self.world);
        Self::position_body(&mut body, &hitbox.position, &hitbox.rotation);

        Phys {
            body: body.handle(),
        }
    }

    #[inline]
    pub fn name_as_ent(&mut self, phys: &Phys, ent_boxed: Box<specs::Entity>) {
        let body = self.world.rigid_body_mut(phys.body).unwrap();
        body.set_name((*ent_boxed).id().to_string().to_owned());
        body.set_user_data(Some(ent_boxed));
    }

    #[inline]
    pub fn do_physics_interact(&self, phys: &Phys) -> bool {
        self.world
            .collider_world()
            .body_colliders(phys.body)
            .next()
            .unwrap()
            .collision_groups()
            .is_member_of(0)
    }

    #[inline]
    pub fn collider(&self, phys: &Phys) -> Option<&Collider<f32>> {
        let part_handle = self.rigid_body(&phys).unwrap().part_handle();
        self.world
            .collider_world()
            .body_part_colliders(part_handle)
            .next()
    }

    #[inline]
    pub fn rigid_body(&self, phys: &Phys) -> Option<&RigidBody<f32>> {
        self.world.rigid_body(phys.body)
    }

    #[inline]
    pub fn location(&self, phys: &Phys) -> Option<&Vector3<f32>> {
        Some(&self.rigid_body(phys)?.position().translation.vector)
    }

    #[inline]
    pub fn euler_vec(&self, phys: &Phys) -> Option<Vector3<f32>> {
        let angles = self.rigid_body(phys)?.position().rotation.euler_angles();
        let eul_vec = glm::vec3(angles.0, angles.1, angles.2);
        Some(eul_vec)
    }

    #[inline]
    pub fn scale(&self, phys: &Phys) -> Option<&Vector3<f32>> {
        Some(
            self.collider(&phys)?
                .shape()
                .as_shape::<Cuboid<f32>>()?
                .half_extents(),
        )
    }

    #[inline]
    pub fn set_location(&mut self, phys: &Phys, location: &Vector3<f32>) {
        let rbd = self.rigid_body_mut(phys).unwrap();
        let mut position = rbd.position().clone();
        position.translation.vector = *location;
        rbd.set_position(position);
    }

    #[inline]
    pub fn position_body(
        body: &mut RigidBody<f32>,
        location: &Vector3<f32>,
        rotation: &Vector3<f32>,
    ) {
        use nalgebra::geometry::{Isometry, Translation, UnitQuaternion};
        body.set_position(Isometry::from_parts(
            Translation::from(*location),
            UnitQuaternion::from_euler_angles(rotation.x, rotation.y, rotation.z),
        ));
    }

    #[inline]
    pub fn set_position(&mut self, phys: &Phys, location: &Vector3<f32>, rotation: &Vector3<f32>) {
        Self::position_body(self.rigid_body_mut(phys).unwrap(), location, rotation);
    }

    #[inline]
    pub fn rigid_body_mut(&mut self, phys: &Phys) -> Option<&mut RigidBody<f32>> {
        self.world.rigid_body_mut(phys.body)
    }

    #[inline]
    pub fn body(&self, phys: &Phys) -> Option<&Body<f32>> {
        self.world.body(phys.body)
    }

    #[inline]
    pub fn body_mut(&mut self, phys: &Phys) -> Option<&mut Body<f32>> {
        self.world.body_mut(phys.body)
    }
}

pub struct Raycaster {
    pub ray: ncollide3d::query::Ray<f32>,
    pub collision_group: ncollide3d::world::CollisionGroups,
}

#[allow(dead_code)]
impl Raycaster {
    #[inline]
    pub fn point_from_camera(screen_coords: &(f32, f32), ls: &LocalState) -> Self {
        use nalgebra::geometry::Point;
        use ncollide3d::{query::Ray, world::CollisionGroups};

        let camera = ls.camera;
        let omnigroup = CollisionGroups::new()
            .with_membership(&[2])
            .with_whitelist(&[0, 1, 2])
            .with_blacklist(&[]);
        let ray = Ray::new(
            Point::from(camera.position + camera.offset),
            glm::unproject(
                &glm::vec3(screen_coords.0, screen_coords.1, -1.0),
                &glm::inverse(&glm::quat_to_mat4(&camera.get_quat())),
                &ls.perspective_projection,
                glm::vec4(0.0, 0.0, ls.frame_width as f32, ls.frame_height as f32),
            ),
        );

        Raycaster {
            ray,
            collision_group: omnigroup,
        }
    }

    #[inline]
    pub fn cast_at_ground<'a>(
        &'a self,
        ps: &'a PhysState,
    ) -> impl Iterator<Item = (&'a Collider<f32>, RayIntersection<f32>)> {
        ps.world
            .collider_world()
            .interferences_with_ray(&self.ray, &self.collision_group)
            .filter(|(collider, _)| collider.body().is_ground())
    }

    #[inline]
    pub fn cast_to_ground_pos(&self, ps: &PhysState) -> Option<glm::TVec3<f32>> {
        ps.world
            .collider_world()
            .interferences_with_ray(&self.ray, &self.collision_group)
            .filter(|(collider, _)| collider.body().is_ground())
            .map(|(_, rayhit)| self.rayhit_pos(&rayhit))
            .next()
    }

    #[inline]
    pub fn rayhit_pos(&self, rayhit: &RayIntersection<f32>) -> glm::TVec3<f32> {
        self.ray.point_at(rayhit.toi).coords
    }
}
