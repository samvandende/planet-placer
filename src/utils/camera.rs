use crate::utils::*;

pub struct Camera {
    pub position: Vec3,
    pub look_dir: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    aspect: f32,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: Mat4,
    position: PackedVec3,
    z_near: f32,
    z_far: f32,
    _padding: u64,
}

impl Camera {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        position: glam::Vec3,
        look_dir: glam::Vec3,
    ) -> Self {
        let aspect = config.width as f32 / config.height as f32;
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("camera_depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Camera {
            position,
            look_dir,
            up: glam::Vec3::Z,
            aspect,
            fov_y: 45f32.to_radians(),
            z_near: 0.01,
            z_far: 100.,
            depth_texture,
            depth_view,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        let new_camera = Camera::new(device, config, self.position, self.look_dir);
        self.aspect = new_camera.aspect;
        self.depth_texture = new_camera.depth_texture;
        self.depth_view = new_camera.depth_view;
    }

    pub fn depth_stencil_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }
}

pub fn uniform_buffer(device: &wgpu::Device) -> Buffer<CameraUniform> {
    device.create_typed_buffer(&TypedBufferDescriptor {
        label: Some("Camera Uniform Buffer"),
        len: 1,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn write_view_projection(
    queue: &wgpu::Queue,
    camera: &Camera,
    uniform_buffer: &Buffer<CameraUniform>,
) {
    let view = glam::Mat4::look_to_rh(Vec3::ZERO, camera.look_dir, camera.up);
    let proj = glam::Mat4::perspective_rh(camera.fov_y, camera.aspect, camera.z_near, camera.z_far);
    let view_proj = proj * view;
    let position = camera.position.into();
    queue.write_typed_buffer(
        uniform_buffer,
        0,
        &[CameraUniform {
            view_proj,
            position,
            z_near: camera.z_near,
            z_far: camera.z_far,
            _padding: 0,
        }],
    );
}

pub fn depth_stencil_state() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}
