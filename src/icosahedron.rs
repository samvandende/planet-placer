use crate::setup;
use crate::utils::*;
use anyhow::Result;

use crate::RADIUS;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: PackedVec3,
    color: Vec3,
    _padding: f32,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Uint32x4, 1 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const PHI: f64 = 1.61803398875; // Golden ratio

#[rustfmt::skip]
pub const VERTICES: &[DVec3] = &[
    DVec3::new(-1.0,  PHI,  0.0),
    DVec3::new( 1.0,  PHI,  0.0),
    DVec3::new(-1.0, -PHI,  0.0),
    DVec3::new( 1.0, -PHI,  0.0),
    DVec3::new( 0.0, -1.0,  PHI),
    DVec3::new( 0.0,  1.0,  PHI),
    DVec3::new( 0.0, -1.0, -PHI),
    DVec3::new( 0.0,  1.0, -PHI),
    DVec3::new( PHI,  0.0, -1.0),
    DVec3::new( PHI,  0.0,  1.0),
    DVec3::new(-PHI,  0.0, -1.0),
    DVec3::new(-PHI,  0.0,  1.0),
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 11, 5,  0, 5, 1,  0, 1, 7,  0, 7, 10,  0, 10, 11,
    1, 5, 9,  5, 11, 4,  11, 10, 2,  10, 7, 6,  7, 1, 8,
    3, 9, 4,  3, 4, 2,  3, 2, 6,  3, 6, 8,  3, 8, 9,
    4, 9, 5,  2, 4, 11,  6, 2, 10,  8, 6, 7,  9, 8, 1,
];

fn subdivide(vertices: &mut Vec<DVec3>, indices: &mut Vec<u16>) {
    let mut new_indices = Vec::new();
    let mut midpoint_cache = std::collections::HashMap::new();

    let midpoint = |a: u16,
                    b: u16,
                    vertices: &mut Vec<DVec3>,
                    cache: &mut std::collections::HashMap<(u16, u16), u16>|
     -> u16 {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&mid) = cache.get(&key) {
            return mid;
        }
        let mid_pos = (vertices[a as usize] + vertices[b as usize]) * 0.5;
        let mid_index = vertices.len() as u16;
        vertices.push(mid_pos.normalize());
        cache.insert(key, mid_index);
        mid_index
    };

    for chunk in indices.chunks_exact(3) {
        let m1 = midpoint(chunk[0], chunk[1], vertices, &mut midpoint_cache);
        let m2 = midpoint(chunk[1], chunk[2], vertices, &mut midpoint_cache);
        let m3 = midpoint(chunk[2], chunk[0], vertices, &mut midpoint_cache);

        new_indices.extend_from_slice(&[
            chunk[0], m1, m3, m1, chunk[1], m2, m3, m2, chunk[2], m1, m2, m3,
        ]);
    }
    *indices = new_indices;
}

pub fn subdivided_icosahedron(subdivisions: usize) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = VERTICES.to_owned();
    let mut indices = INDICES.to_owned();
    for vertex in &mut vertices {
        *vertex = vertex.normalize();
    }
    for _ in 0..subdivisions {
        subdivide(&mut vertices, &mut indices);
    }
    let vertices = vertices
        .into_iter()
        .map(|vertex| Vertex {
            position: (vertex * RADIUS).into(),
            color: Vec3::ONE,
            _padding: 0.,
        })
        .collect();
    (vertices, indices)
}

pub fn vertex_buffer(device: &wgpu::Device, vertices: &[Vertex]) -> Buffer<Vertex> {
    device.create_typed_buffer_init(&TypedBufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: vertices,
        usage: wgpu::BufferUsages::VERTEX,
    })
}

pub fn index_buffer(device: &wgpu::Device, indices: &[u16]) -> Buffer<u16> {
    device.create_typed_buffer_init(&TypedBufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: indices,
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub struct Icosahedron {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u16>,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Icosahedron {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_uniform: &Buffer<camera::CameraUniform>,
        subdivisions: usize,
    ) -> Result<Self> {
        let (vertices, indices) = subdivided_icosahedron(subdivisions);
        let vertex_buffer = vertex_buffer(device, &vertices);
        let index_buffer = index_buffer(device, &indices);

        let shader = setup::shader(device, "shaders/simple_3d.wgsl")?;

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
                label: Some("Triangle Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
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
                polygon_mode: wgpu::PolygonMode::Line,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(camera::depth_stencil_state()),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Icosahedron {
            vertex_buffer,
            index_buffer,
            bind_group,
            render_pipeline,
        })
    }
}

pub fn render(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera: &camera::Camera,
    triangle: &Icosahedron,
) -> Result<(), wgpu::SurfaceError> {
    let output = surface.get_current_texture()?;

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(camera.depth_stencil_attachment()),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&triangle.render_pipeline);
        render_pass.set_bind_group(0, &triangle.bind_group, &[]);
        render_pass.set_typed_vertex_buffer(0, &triangle.vertex_buffer);
        render_pass.set_typed_index_buffer(&triangle.index_buffer);
        render_pass.draw_indexed(0..triangle.index_buffer.len as _, 0, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
