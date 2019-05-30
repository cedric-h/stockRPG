use super::draw_data::*;
use super::helper;
use crate::prelude::*;
use image::RgbaImage;
use nalgebra::{Point3};
use wgpu::{Color, CommandEncoder as Encoder, Device, SwapChainDescriptor, TextureView};

pub struct GameRenderer {
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    depth: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    pub clear_color: Color,
}

impl GameRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::D32Float;

    pub fn init(texels: RgbaImage, sc_desc: &SwapChainDescriptor, device: &mut Device) -> Self {
        use std::mem;

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        //index buffer!
        let index_data: Vec<u16> = vec![0, 1, 2, 2, 1, 3];
        let index_buf = device
            .create_buffer_mapped(index_data.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&index_data);

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer,
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture,
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let texture_extent = wgpu::Extent3d {
            width: texels.width(),
            height: texels.height(),
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::TRANSFER_DST,
        });
        let texture_view = texture.create_default_view();
        let temp_buf = device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsage::TRANSFER_SRC)
            .fill_from_slice(&texels);
        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * texels.width(),
                image_height: texels.height(),
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            texture_extent,
        );

        // Create other resources
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });
        let mx_total = glm::TMat4::<f32>::identity().data;
        let mx_ref: &[f32] = mx_total.as_ref();
        let uniform_buf = device
            .create_buffer_mapped(
                64,
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_DST,
            )
            .fill_from_slice(&helper::cast_slice(mx_ref));

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create the render pipeline
        let vs_bytes = helper::load_glsl(include_str!("shader.vert"), helper::ShaderStage::Vertex);
        let fs_bytes =
            helper::load_glsl(include_str!("shader.frag"), helper::ShaderStage::Fragment);
        let vs_module = device.create_shader_module(&vs_bytes);
        let fs_module = device.create_shader_module(&fs_bytes);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::PipelineStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::PipelineStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            },
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: Self::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float4,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 4 * 4,
                        shader_location: 1,
                    },
                ],
            }],
            sample_count: 1,
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        // Done
        let init_command_buf = init_encoder.finish();
        device.get_queue().submit(&[init_command_buf]);
        Self {
            index_buf,
            index_count: index_data.len(),
            bind_group,
            uniform_buf,
            depth: depth_texture.create_default_view(),
            pipeline,
            clear_color: Color::BLACK,
        }
    }

    fn get_draw_data(&mut self, world: &specs::World) -> DrawData {
        use specs::Join;

        let physes = world.read_storage::<Phys>();
        let cam_fs = world.read_storage::<CameraFocus>();
        let appears = world.read_storage::<Appearance>();
        let ls = world.read_resource::<LocalState>();
        let ps = world.read_resource::<PhysState>();

        let fill = &cam_fs
            .join()
            .next()
            .map(|cf| cf.background_color)
            .unwrap_or([0.1, 0.2, 0.3, 1.0]);
        self.clear_color = Color {
            r: fill[0],
            g: fill[1],
            b: fill[2],
            a: fill[3],
        };

        DrawData {
            view_projection: ls.perspective_projection * ls.camera.view_matrix,
            vertexes: (&appears, &physes)
                .join()
                .map(|(Appearance { size, uvs: uv }, phys)| {
                    #[cfg_attr(rustfmt, rustfmt_skip)]
                    let vertex_data = vec![
                        //top left
                        ([-size[0], 0.0, -size[1]], [uv[0], uv[3]]),
                        //bottom left
                        ([-size[0], 0.0,  size[1]], [uv[0], uv[1]]),
                        //top right
                        ([ size[0], 0.0, -size[1]], [uv[2], uv[3]]),
                        //bottom right
                        ([ size[0], 0.0,  size[1]], [uv[2], uv[1]]),
                    ];

                    let iso = ps.rigid_body(phys).unwrap().position();

                    vertex_data
                        .iter()
                        .map(|(pos_arr, uv)| vertex(iso * Point3::from(*pos_arr), uv))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn resize(&mut self, sc_desc: &SwapChainDescriptor, device: &mut Device) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        self.depth = depth_texture.create_default_view();
    }
    pub fn render(
        &mut self,
        world: &specs::World,
        device: &mut Device,
        encoder: &mut Encoder,
        view: &TextureView,
    ) -> Result<(), String> {
        let dd = self.get_draw_data(world);

        {
            let mx_ref: &[f32] = dd.view_projection.data.as_ref();

            let temp_buf = device
                .create_buffer_mapped(64, wgpu::BufferUsage::TRANSFER_SRC)
                .fill_from_slice(&helper::cast_slice(mx_ref));

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
            encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.uniform_buf, 0, 64);
            device.get_queue().submit(&[encoder.finish()]);
        }

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: self.clear_color,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                clear_stencil: 0,
            }),
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_index_buffer(&self.index_buf, 0);

        for vertex_data in dd.vertexes.iter() {
            let vertex_buf = device
                .create_buffer_mapped(vertex_data.len(), wgpu::BufferUsage::VERTEX)
                .fill_from_slice(&vertex_data);
            rpass.set_vertex_buffers(&[(&vertex_buf, 0)]);
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        Ok(())
    }
}
