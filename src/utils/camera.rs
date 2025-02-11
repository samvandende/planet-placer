use crate::utils::*;

pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

pub fn uniform_buffer(device: &wgpu::Device) -> Buffer<glam::Mat4> {
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
    uniform_buffer: &Buffer<glam::Mat4>,
) {
    let view = glam::Mat4::look_at_rh(camera.eye, camera.target, camera.up);
    let proj = glam::Mat4::perspective_rh(camera.fov_y, camera.aspect, camera.z_near, camera.z_far);
    let view_projection = proj * view;
    queue.write_typed_buffer(uniform_buffer, 0, &[view_projection]);
}
