use anyhow::Result;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

pub type WindowSize = winit::dpi::PhysicalSize<u32>;

pub fn window(window_target: &EventLoopWindowTarget<()>) -> Result<Window> {
    Ok(WindowBuilder::new().build(window_target)?)
}

pub fn instance() -> wgpu::Instance {
    wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    })
}

pub unsafe fn surface(
    instance: &wgpu::Instance,
    window: &Window,
) -> Result<wgpu::Surface<'static>> {
    let target = wgpu::SurfaceTargetUnsafe::from_window(window)?;
    Ok(instance.create_surface_unsafe(target)?)
}

pub async fn adapter_async(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'static>,
) -> Option<wgpu::Adapter> {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
}

pub fn adapter(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'static>,
) -> Option<wgpu::Adapter> {
    pollster::block_on(adapter_async(instance, surface))
}

pub async fn device_queue_async(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
    Ok(adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        )
        .await?)
}

pub fn device_queue(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(device_queue_async(adapter))
}

pub fn surface_config(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
) -> wgpu::SurfaceConfiguration {
    let caps = surface.get_capabilities(adapter);
    let surface_format = caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(caps.formats[0]);

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: 0,
        height: 0,
        present_mode: caps.present_modes[0],
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

pub fn configure_surface(
    surface: &wgpu::Surface<'static>,
    device: &wgpu::Device,
    config: &mut wgpu::SurfaceConfiguration,
    size: WindowSize,
) -> bool {
    if size.width == 0 || size.height == 0 {
        return false;
    }

    config.width = size.width;
    config.height = size.height;
    surface.configure(device, config);
    true
}

pub fn shader(device: &wgpu::Device, file: impl AsRef<Path>) -> Result<wgpu::ShaderModule> {
    let mut shader_file = File::open(file.as_ref())?;
    let mut shader_contents = String::new();
    shader_file.read_to_string(&mut shader_contents)?;

    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: file.as_ref().file_name().and_then(OsStr::to_str),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_contents)),
    }))
}

pub mod triangle {
    use super::shader;
    use anyhow::Result;
    use wgpu::util::DeviceExt;

    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    struct Vertex {
        position: [f32; 3],
        color: [f32; 3],
    }

    impl Vertex {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        fn desc() -> wgpu::VertexBufferLayout<'static> {
            use std::mem;

            wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &Self::ATTRIBS,
            }
        }
    }

    const VERTICES: &[Vertex] = &[
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];

    pub fn vertex_buffer(device: &wgpu::Device) -> (wgpu::Buffer, usize) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        (buffer, VERTICES.len())
    }

    pub fn render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Result<wgpu::RenderPipeline> {
        let shader = shader(device, "shaders/triangle.wgsl")?;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle Render Pipeline Layout"),
                bind_group_layouts: &[],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(render_pipeline)
    }
}
