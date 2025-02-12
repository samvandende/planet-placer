use utils::*;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

mod icosahedron;
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

    let mut camera = camera::Camera::new(
        &device,
        &config,
        glam::vec3(0., 0., 1.),
        glam::vec3(0., -1., -2.).normalize(),
    );
    let camera_uniform = camera::uniform_buffer(&device);

    let triangle = icosahedron::Icosahedron::new(&device, &config, &camera_uniform, 4)?;

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

                update(start.elapsed().as_secs_f32(), &mut camera);
                camera::write_view_projection(&queue, &camera, &camera_uniform);
                match icosahedron::render(&surface, &device, &queue, &camera, &triangle) {
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

fn update(t: f32, camera: &mut camera::Camera) {
    let (x, y) = (1.3 * t).sin_cos();
    camera.position.x = 4. * x;
    camera.position.y = 4. * y;
    camera.look_dir = -camera.position.normalize()
}
