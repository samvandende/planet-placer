use std::collections::HashSet;

use crate::setup;
use crate::utils::*;
use anyhow::Result;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_pcg::Pcg32;
use tectonic_plates::TectonicPlateClassification;

use crate::RADIUS;
mod regions;
use regions::Region;
mod tectonic_plates;

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

impl Vertex {
    #[rustfmt::skip]
    fn from_region(region: &Region, classification: TectonicPlateClassification) -> [Self; 3] {
        let color = match classification {
            TectonicPlateClassification::Continental => vec3(0., 1., 0.),
            TectonicPlateClassification::Oceanic => vec3(0., 0., 1.),
        };
        [
            Vertex { position: region.corners[0].into(), color, _padding: 0. },
            Vertex { position: region.corners[1].into(), color, _padding: 0. },
            Vertex { position: region.corners[2].into(), color, _padding: 0. },
        ]
    }
}

pub fn build_planet() -> (Vec<Vertex>, Vec<u16>) {
    let mut rng = Pcg32::seed_from_u64(1);
    let regions = regions::create_regions(5);
    let tectonic_plates = tectonic_plates::cluster_regions(&mut rng, &regions, 40);

    let mut vertices = vec![];
    for plate in &tectonic_plates {
        for region_index in &plate.contained_regions {
            let region = &regions[*region_index];
            let verts = Vertex::from_region(region, plate.classification);
            for v in verts {
                vertices.push(v);
            }
        }
    }
    let indices = (0..vertices.len() as u16).collect();

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

pub struct Planet {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u16>,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Planet {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_uniform: &Buffer<camera::CameraUniform>,
    ) -> Result<Self> {
        let (vertices, indices) = build_planet();

        let vertex_buffer = vertex_buffer(device, &vertices);
        let index_buffer = index_buffer(device, &indices);

        let shader = setup::shader(device, "shaders/planet.wgsl")?;

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
                polygon_mode: wgpu::PolygonMode::Fill,
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

        Ok(Planet {
            vertex_buffer,
            index_buffer,
            bind_group,
            render_pipeline,
        })
    }
}

pub fn render(render_pass: &mut wgpu::RenderPass, planet: &Planet) {
    render_pass.set_pipeline(&planet.render_pipeline);
    render_pass.set_bind_group(0, &planet.bind_group, &[]);
    render_pass.set_typed_vertex_buffer(0, &planet.vertex_buffer);
    render_pass.set_typed_index_buffer(&planet.index_buffer);
    render_pass.draw_indexed(0..planet.index_buffer.len as _, 0, 0..1);
}
