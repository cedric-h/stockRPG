use crate::prelude::*;
use nalgebra::{Isometry3, Point3};
use specs::{Join, World};

pub struct SpritesheetDimensions {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
pub struct SpritesheetVertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

impl SpritesheetVertex {
    pub fn new(iso: &Isometry3<f32>, loc: &[f32; 3], tc: &[f32; 2]) -> Self {
        let pnt = iso * Point3::from(*loc);
        Self {
            _pos: [pnt.x, pnt.y, pnt.z, 1.0],
            _tex_coord: [tc[0], tc[1]],
        }
    }

    pub fn get_from_ecs(world: &World) -> Vec<Vec<Self>> {
        let physes = world.read_storage::<Phys>();
        let appears = world.read_storage::<Appearance>();
        let ps = world.read_resource::<PhysState>();

        (&appears, &physes)
            .join()
            .map(|(Appearance { size, uvs: uv }, phys)| {
                let iso = ps.rigid_body(phys).unwrap().position();

                #[cfg_attr(rustfmt, rustfmt_skip)]
                vec![
                    //top left
                    ([-size[0], 0.0, -size[1]], [uv[0], uv[3]]),
                    //bottom left
                    ([-size[0], 0.0,  size[1]], [uv[0], uv[1]]),
                    //top right
                    ([ size[0], 0.0, -size[1]], [uv[2], uv[3]]),
                    //bottom right
                    ([ size[0], 0.0,  size[1]], [uv[2], uv[1]]),
                ]
                .iter()
                .map(|(loc, uv)| Self::new(iso, loc, uv) )
                .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Copy)]
pub struct BoxOutlineVertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
    _hole_size: [f32; 2],
    _rgb: [f32; 3],
}

impl BoxOutlineVertex {
    pub fn from_ss_vert(ss_vert: SpritesheetVertex, hole_size: [f32; 2], rgb: [f32; 3]) -> Self {
        Self {
            //from ss
            _pos: ss_vert._pos,
            _tex_coord: ss_vert._tex_coord,

            //from parameters
            _hole_size: hole_size,
            _rgb: rgb,
        }
    }

    pub fn get_from_ecs(world: &World) -> Vec<Vec<Self>> {
        #[inline]
        fn fade_color_arr(arr: [f32; 3], num: f32) -> [f32; 3] {
            [
                (arr[0] + num).min(1.0),
                (arr[1] + num).min(1.0),
                (arr[2] + num).min(1.0),
            ]
        }

        let physes = world.read_storage::<Phys>();
        let outlines = world.read_storage::<BoxOutline>();
        let ps = world.read_resource::<PhysState>();

        let mut all_outlines = Vec::new();

        for (phys, bo) in (&physes, &outlines).join() {
            let s = ps.scale(phys).unwrap();
            let iso = ps.rigid_body(phys).unwrap().position();

            //input color
            let npt = bo.color;
            //edge fade
            let fade = bo.fade;

            #[cfg_attr(rustfmt, rustfmt_skip)]
            let front_side = vec![
                //top left
                ([-s.x, s.y/2.0, -s.z], [-s.x/2.0,  s.z/2.0]),
                //bottom left
                ([-s.x, s.y/2.0,  s.z], [-s.x/2.0, -s.z/2.0]),
                //top right
                ([ s.x, s.y/2.0, -s.z], [ s.x/2.0,  s.z/2.0]),
                //bottom right
                ([ s.x, s.y/2.0,  s.z], [ s.x/2.0, -s.z/2.0]),
            ]
            .iter()
            .enumerate()
            .map(|(index, (loc, uv))| (*loc, *uv, fade_color_arr(npt, fade[index])))
            .collect::<Vec<_>>();

            let back_side = {
                let mut b = front_side.clone();
                let last_color = b[3].2.clone();
                //shift the colors around
                b.iter_mut().fold(last_color, |acc, vert| {
                    let old_color = vert.2;
                    vert.2 = acc;
                    old_color
                });
                b.iter_mut().for_each(|(loc, _, _)| loc[1] -= s.y);
                b
            };

            let hole_size = [s.x / 2.0 - 1.0 / 48.0, s.z / 2.0 - 1.0 / 48.0];

            for vertex_data in [front_side, back_side].iter() {
                all_outlines.push(
                    vertex_data
                        .iter()
                        .map(|(loc, uv, rgb)| (SpritesheetVertex::new(iso, loc, uv), rgb))
                        .map(|(ss_vert, rgb)| Self::from_ss_vert(ss_vert, hole_size, *rgb))
                        .collect::<Vec<_>>(),
                );
            }
        }

        all_outlines
    }
}
