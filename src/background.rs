use crate::{setup, utils::*};
use anyhow::Result;
use camera::{Camera, CameraUniform};

pub fn vec3_vertex_desc() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;

    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vec3>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRIBS,
    }
}

pub fn build_near_field_quad(camera: &Camera) -> [Vec3; 4] {
    let look_dir = camera.look_dir;
    assert!(look_dir.is_normalized(), "look_dir not normalized");

    let znear = camera.z_near * 1.01;
    let half_height = (camera.fov_y * 0.5).tan() * znear;
    let half_width = half_height * camera.aspect_ratio();

    let near_center = look_dir * znear;
    let right = look_dir.cross(camera.up).normalize();
    let up = right.cross(look_dir);

    let top_left = near_center + up * half_height - right * half_width;
    let top_right = near_center + up * half_height + right * half_width;
    let bottom_left = near_center - up * half_height - right * half_width;
    let bottom_right = near_center - up * half_height + right * half_width;

    [top_left, top_right, bottom_left, bottom_right]
}

pub fn create_near_field_quad_vertex_buffer(device: &wgpu::Device) -> Buffer<Vec3> {
    device.create_typed_buffer(&TypedBufferDescriptor {
        label: Some("near_field_quad_vertex_buffer"),
        len: 4,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn create_near_field_quad_index_buffer(device: &wgpu::Device) -> Buffer<u16> {
    device.create_typed_buffer_init(&TypedBufferInitDescriptor {
        label: Some("near_field_quad_index_buffer"),
        contents: &[0, 2, 1, 1, 2, 3],
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub struct Background {
    vertex_buffer: Buffer<Vec3>,
    index_buffer: Buffer<u16>,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Background {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_uniform: &Buffer<CameraUniform>,
    ) -> Result<Self> {
        let vertex_buffer = create_near_field_quad_vertex_buffer(device);
        let index_buffer = create_near_field_quad_index_buffer(device);

        let shader = setup::shader(device, "shaders/background.wgsl")?;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Background Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vec3_vertex_desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Background {
            vertex_buffer,
            index_buffer,
            bind_group,
            render_pipeline,
        })
    }

    pub fn update_screen_quad(&self, queue: &wgpu::Queue, camera: &Camera) {
        queue.write_typed_buffer(&self.vertex_buffer, 0, &build_near_field_quad(camera));
    }
}

pub fn render(render_pass: &mut wgpu::RenderPass, background: &Background) {
    render_pass.set_pipeline(&background.render_pipeline);
    render_pass.set_bind_group(0, &background.bind_group, &[]);
    render_pass.set_typed_vertex_buffer(0, &background.vertex_buffer);
    render_pass.set_typed_index_buffer(&background.index_buffer);
    render_pass.draw_indexed(0..background.index_buffer.len as _, 0, 0..1);
}
