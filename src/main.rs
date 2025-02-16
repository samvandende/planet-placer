use utils::*;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

mod background;
mod planet;
mod setup;
mod utils;

const RADIUS: f64 = 1.0;

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

    let mut camera = camera::Camera::new(
        &device,
        &config,
        dvec3(0., 0., RADIUS),
        vec3(0., -1., -2.).normalize(),
    );
    let camera_uniform = camera::uniform_buffer(&device);

    let background = background::Background::new(&device, &config, &camera_uniform)?;
    let planet = planet::Planet::new(&device, &config, &camera_uniform)?;

    let start = std::time::Instant::now();
    event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Resized(new_size) => {
                surface_configured =
                    setup::configure_surface(&surface, &device, &mut config, *new_size);
                camera.resize(&device, &config);
            }
            WindowEvent::RedrawRequested => {
                window.request_redraw();

                if !surface_configured {
                    return;
                }

                update(start.elapsed().as_secs_f64(), &mut camera);
                camera::write_view_projection(&queue, &camera, &camera_uniform);
                background.update_screen_quad(&queue, &camera);

                match render(&surface, &device, &queue, &camera, &background, &planet) {
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

fn update(t: f64, camera: &mut camera::Camera) {
    let (x, y) = (0.1 * t).sin_cos();
    camera.position.x = 4. * RADIUS * x;
    camera.position.y = 4. * RADIUS * y;
    camera.look_dir = -camera.position.normalize().as_vec3()
}

fn render(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera: &camera::Camera,
    background: &background::Background,
    planet: &planet::Planet,
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
                        r: 0.01,
                        g: 0.01,
                        b: 0.01,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        background::render(&mut render_pass, background);
    }

    // {
    //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //         label: Some("Render Pass"),
    //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //             view: &view,
    //             resolve_target: None,
    //             ops: wgpu::Operations {
    //                 load: wgpu::LoadOp::Load,
    //                 store: wgpu::StoreOp::Store,
    //             },
    //         })],
    //         depth_stencil_attachment: Some(camera.depth_stencil_attachment()),
    //         occlusion_query_set: None,
    //         timestamp_writes: None,
    //     });

    //     planet::render(&mut render_pass, planet);
    // }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
