use utils::*;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

mod setup;
mod utils;

pub fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = setup::window(&event_loop)?;
    let instance = setup::instance();
    let surface = unsafe { setup::surface(&instance, &window) }?;
    let adapter = setup::adapter(&instance, &surface).unwrap();
    let (device, queue) = setup::device_queue(&adapter)?;
    let mut config = setup::surface_config(&surface, &adapter);
    let mut surface_configured =
        setup::configure_surface(&surface, &device, &mut config, window.inner_size());
    let triangle_pipeline = setup::triangle::render_pipeline(&device, &config)?;
    let triangle_vertex_buffer = setup::triangle::vertex_buffer(&device);
    let triangle_index_buffer = setup::triangle::index_buffer(&device);

    event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Resized(new_size) => {
                surface_configured =
                    setup::configure_surface(&surface, &device, &mut config, *new_size);
            }
            WindowEvent::RedrawRequested => {
                window.request_redraw();

                if !surface_configured {
                    return;
                }

                match render_triangle(
                    &surface,
                    &device,
                    &queue,
                    &triangle_pipeline,
                    &triangle_vertex_buffer,
                    &triangle_index_buffer,
                ) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        surface_configured = setup::configure_surface(
                            &surface,
                            &device,
                            &mut config,
                            window.inner_size(),
                        );
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                        log::error!("OutOfMemory");
                        control_flow.exit();
                        return;
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout");
                        return;
                    }
                };
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => control_flow.exit(),
            _ => {}
        },
        _ => {}
    })?;

    Ok(())
}

fn render_triangle(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    triangle_pipeline: &wgpu::RenderPipeline,
    vertex_buffer: &Buffer<setup::triangle::Vertex>,
    index_buffer: &Buffer<u16>,
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
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(triangle_pipeline);
        render_pass.set_typed_vertex_buffer(0, vertex_buffer);
        render_pass.set_typed_index_buffer(index_buffer);
        render_pass.draw_indexed(0..index_buffer.len as _, 0, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
