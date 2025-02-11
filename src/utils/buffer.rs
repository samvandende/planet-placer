use std::marker::PhantomData;
use wgpu::util::{DeviceExt, RenderEncoder};

pub struct TypedBufferDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub len: usize,
    pub usage: wgpu::BufferUsages,
    pub mapped_at_creation: bool,
}

/// Describes a [Buffer](crate::Buffer) when allocating.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypedBufferInitDescriptor<'a, T: bytemuck::Pod + bytemuck::Zeroable> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: wgpu::Label<'a>,
    /// Contents of a buffer on creation.
    pub contents: &'a [T],
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: wgpu::BufferUsages,
}

pub struct Buffer<T> {
    buffer: wgpu::Buffer,
    pub len: usize,
    _type: PhantomData<T>,
}

impl<T> std::ops::Deref for Buffer<T> {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

pub trait BufferDeviceExt<'a, T: bytemuck::Pod + bytemuck::Zeroable> {
    fn create_typed_buffer(&self, desc: &TypedBufferDescriptor) -> Buffer<T>;
    fn create_typed_buffer_init(&self, desc: &TypedBufferInitDescriptor<'a, T>) -> Buffer<T>;
}

pub trait BufferQueueExt<T: bytemuck::Pod + bytemuck::Zeroable> {
    fn write_typed_buffer(&self, buffer: &Buffer<T>, offset: u64, data: &[T]) {}
}

pub trait BufferVertexRenderPassExt<T> {
    fn set_typed_vertex_buffer(&mut self, slot: u32, buffer: &Buffer<T>);
}

pub trait BufferIndexRenderPassExt<T> {
    fn set_typed_index_buffer(&mut self, buffer: &Buffer<T>);
}

impl<'a, T: Sized + bytemuck::Pod + bytemuck::Zeroable> BufferDeviceExt<'a, T> for wgpu::Device {
    fn create_typed_buffer(&self, desc: &TypedBufferDescriptor) -> Buffer<T> {
        let len = desc.len;
        let desc = wgpu::BufferDescriptor {
            label: desc.label,
            size: desc.len as u64 * std::mem::size_of::<T>() as u64,
            usage: desc.usage,
            mapped_at_creation: desc.mapped_at_creation,
        };
        let buffer = self.create_buffer(&desc);
        Buffer {
            buffer,
            len,
            _type: Default::default(),
        }
    }

    fn create_typed_buffer_init(&self, desc: &TypedBufferInitDescriptor<'a, T>) -> Buffer<T> {
        let len = desc.contents.len();
        let contents = bytemuck::cast_slice(&desc.contents);
        let desc = wgpu::util::BufferInitDescriptor {
            label: desc.label,
            contents,
            usage: desc.usage,
        };
        let buffer = <wgpu::Device as DeviceExt>::create_buffer_init(self, &desc);
        Buffer {
            buffer,
            len,
            _type: Default::default(),
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> BufferQueueExt<T> for wgpu::Queue {
    fn write_typed_buffer(&self, buffer: &Buffer<T>, offset: u64, data: &[T]) {
        self.write_buffer(&buffer.buffer, offset, bytemuck::cast_slice(data));
    }
}

impl<'a, T: bytemuck::Pod + bytemuck::Zeroable> BufferVertexRenderPassExt<T>
    for wgpu::RenderPass<'a>
{
    fn set_typed_vertex_buffer(&mut self, slot: u32, buffer: &Buffer<T>) {
        self.set_vertex_buffer(slot, buffer.buffer.slice(..));
    }
}

impl<'a> BufferIndexRenderPassExt<u16> for wgpu::RenderPass<'a> {
    fn set_typed_index_buffer(&mut self, buffer: &Buffer<u16>) {
        self.set_index_buffer(buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
    }
}

impl<'a> BufferIndexRenderPassExt<u32> for wgpu::RenderPass<'a> {
    fn set_typed_index_buffer(&mut self, buffer: &Buffer<u32>) {
        self.set_index_buffer(buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);
    }
}
