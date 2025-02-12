use anyhow::Result;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

pub type WindowSize = winit::dpi::PhysicalSize<u32>;

const FEATURES: wgpu::Features = wgpu::Features::POLYGON_MODE_LINE;

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
                required_features: FEATURES,
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
