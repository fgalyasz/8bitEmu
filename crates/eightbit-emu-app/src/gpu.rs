use std::fmt;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use eightbit_emu_core::{Frame, Look, PaletteKind, centered_viewport, integer_scale};
use wgpu::util::DeviceExt;
use winit::window::Window;

const SHADER: &str = include_str!("present.wgsl");

#[derive(Debug)]
pub struct GpuError {
    message: String,
}

impl fmt::Display for GpuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl GpuError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    width: u32,
    height: u32,
    look: u32,
    picture_w: u32,
    origin_x: u32,
    origin_y: u32,
    picture_h: u32,
    pad1: u32,
    palette: [[u32; 4]; 16],
}

pub struct Present {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
}

pub fn render_frame(frame: &Frame, look: Look) -> Result<Vec<u8>, GpuError> {
    let present = Present::new(wgpu::TextureFormat::Rgba8Unorm)?;
    present.readback(frame, look, 1, 0, 0)
}

impl Present {
    pub fn new(format: wgpu::TextureFormat) -> Result<Self, GpuError> {
        let instance = wgpu::Instance::default();
        Self::from_instance(&instance, format, &adapter_options(None))
    }

    pub fn with_surface(
        window: Arc<Window>,
        format: wgpu::TextureFormat,
    ) -> Result<(Self, wgpu::Surface<'static>), GpuError> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).map_err(gpu_error)?;
        let present = Self::from_instance(&instance, format, &adapter_options(Some(&surface)))?;
        Ok((present, surface))
    }

    fn from_instance(
        instance: &wgpu::Instance,
        format: wgpu::TextureFormat,
        options: &wgpu::RequestAdapterOptions<'_, '_>,
    ) -> Result<Self, GpuError> {
        let adapter = block(instance.request_adapter(options))?;
        let (device, queue) = block(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
        let layout = bind_layout(&device);
        let pipeline = pipeline(&device, &layout, format);
        Ok(Self {
            device,
            queue,
            pipeline,
            layout,
            format,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn present_surface(&self, texture: wgpu::SurfaceTexture) {
        self.queue.present(texture);
    }

    pub fn clear_surface(&self, texture: wgpu::SurfaceTexture) -> Result<(), GpuError> {
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("eightbit-emu-clear"),
                color_attachments: &[Some(color_attachment(&view))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.queue.submit([encoder.finish()]);
        self.present_surface(texture);
        Ok(())
    }

    pub fn readback(
        &self,
        frame: &Frame,
        look: Look,
        scale: u32,
        origin_x: u32,
        origin_y: u32,
    ) -> Result<Vec<u8>, GpuError> {
        let width = u32::from(frame.width) * scale;
        let height = u32::from(frame.height) * scale;
        let texture = target_texture(&self.device, width, height, self.format);
        self.draw_to(&texture, frame, look, width, height, origin_x, origin_y)?;
        copy_texture(&self.device, &self.queue, &texture, width, height)
    }

    pub fn draw_to(
        &self,
        texture: &wgpu::Texture,
        frame: &Frame,
        look: Look,
        picture_w: u32,
        picture_h: u32,
        origin_x: u32,
        origin_y: u32,
    ) -> Result<(), GpuError> {
        let group = self.bind_group(frame, look, picture_w, picture_h, origin_x, origin_y);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        encode(&self.device, &self.queue, &self.pipeline, &group, &view);
        Ok(())
    }

    fn bind_group(
        &self,
        frame: &Frame,
        look: Look,
        picture_w: u32,
        picture_h: u32,
        origin_x: u32,
        origin_y: u32,
    ) -> wgpu::BindGroup {
        let uniforms = uniform_buffer(&self.device, frame, look, picture_w, picture_h, origin_x, origin_y);
        let planes = plane_texture(&self.device, &self.queue, frame);
        let extra = extra_texture(&self.device, &self.queue, frame);
        let plane_view = planes.create_view(&wgpu::TextureViewDescriptor::default());
        let extra_view = extra.create_view(&wgpu::TextureViewDescriptor::default());
        let entries = [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&plane_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&extra_view),
            },
        ];
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("eightbit-emu-present"),
            layout: &self.layout,
            entries: &entries,
        })
    }
}

fn block<T>(
    future: impl std::future::Future<Output = Result<T, impl fmt::Display>>,
) -> Result<T, GpuError> {
    pollster::block_on(future).map_err(gpu_error)
}

fn gpu_error(error: impl fmt::Display) -> GpuError {
    GpuError {
        message: error.to_string(),
    }
}

fn adapter_options<'a>(
    surface: Option<&'a wgpu::Surface<'a>>,
) -> wgpu::RequestAdapterOptions<'a, 'a> {
    wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: surface,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }
}

fn bind_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("eightbit-emu-present"),
        entries: &[uniform_entry(), texture_entry(1), texture_entry(2)],
    })
}

fn uniform_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("eightbit-emu-present"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("eightbit-emu-present"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    let target = color_target(format);
    let targets = [Some(target)];
    device.create_render_pipeline(&pipeline_desc(&shader, &pipeline_layout, &targets))
}

fn color_target(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    }
}

fn pipeline_desc<'a>(
    shader: &'a wgpu::ShaderModule,
    layout: &'a wgpu::PipelineLayout,
    targets: &'a [Option<wgpu::ColorTargetState>],
) -> wgpu::RenderPipelineDescriptor<'a> {
    wgpu::RenderPipelineDescriptor {
        label: Some("eightbit-emu-present"),
        layout: Some(layout),
        vertex: vertex_state(shader),
        fragment: Some(fragment_state(shader, targets)),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    }
}

fn vertex_state(shader: &wgpu::ShaderModule) -> wgpu::VertexState<'_> {
    wgpu::VertexState {
        module: shader,
        entry_point: Some("vs_main"),
        buffers: &[],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    }
}

fn fragment_state<'a>(
    shader: &'a wgpu::ShaderModule,
    targets: &'a [Option<wgpu::ColorTargetState>],
) -> wgpu::FragmentState<'a> {
    wgpu::FragmentState {
        module: shader,
        entry_point: Some("fs_main"),
        targets,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    }
}

fn uniform_buffer(
    device: &wgpu::Device,
    frame: &Frame,
    look: Look,
    picture_w: u32,
    picture_h: u32,
    origin_x: u32,
    origin_y: u32,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("eightbit-emu-uniforms"),
        contents: bytemuck::bytes_of(&uniforms(frame, look, picture_w, picture_h, origin_x, origin_y)),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

fn uniforms(
    frame: &Frame,
    look: Look,
    picture_w: u32,
    picture_h: u32,
    origin_x: u32,
    origin_y: u32,
) -> Uniforms {
    Uniforms {
        width: u32::from(frame.width),
        height: u32::from(frame.height),
        look: look.shader_id(),
        picture_w,
        origin_x,
        origin_y,
        picture_h,
        pad1: 0,
        palette: palette_words(frame.palette),
    }
}

fn palette_words(kind: PaletteKind) -> [[u32; 4]; 16] {
    let colors = kind.colors();
    let mut words = [[0; 4]; 16];
    let mut index = 0;
    while index < 16 {
        words[index] = [
            u32::from(colors[index].r),
            u32::from(colors[index].g),
            u32::from(colors[index].b),
            255,
        ];
        index += 1;
    }
    words
}

fn plane_texture(device: &wgpu::Device, queue: &wgpu::Queue, frame: &Frame) -> wgpu::Texture {
    rgba_texture(device, queue, frame, pack_planes(frame))
}

fn extra_texture(device: &wgpu::Device, queue: &wgpu::Queue, frame: &Frame) -> wgpu::Texture {
    rgba_texture(device, queue, frame, pack_extra(frame))
}

fn rgba_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame: &Frame,
    bytes: Vec<u8>,
) -> wgpu::Texture {
    let texture = device.create_texture(&texture_desc(frame));
    queue.write_texture(
        texture.as_image_copy(),
        &bytes,
        texture_layout(frame),
        texture_size(frame),
    );
    texture
}

fn texture_desc(frame: &Frame) -> wgpu::TextureDescriptor<'_> {
    wgpu::TextureDescriptor {
        label: Some("eightbit-emu-plane"),
        size: texture_size(frame),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    }
}

fn texture_size(frame: &Frame) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: u32::from(frame.width),
        height: u32::from(frame.height),
        depth_or_array_layers: 1,
    }
}

fn texture_layout(frame: &Frame) -> wgpu::TexelCopyBufferLayout {
    wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(u32::from(frame.width) * 4),
        rows_per_image: Some(u32::from(frame.height)),
    }
}

fn pack_planes(frame: &Frame) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut index = 0;
    while index < frame.index.len() {
        push_plane(&mut bytes, frame, index);
        index += 1;
    }
    bytes
}

fn push_plane(bytes: &mut Vec<u8>, frame: &Frame, index: usize) {
    bytes.push(frame.index[index]);
    bytes.push(frame.luma[index]);
    bytes.push(frame.sprite[index]);
    bytes.push(frame.sprite_on[index]);
}

fn pack_extra(frame: &Frame) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut index = 0;
    while index < frame.index.len() {
        push_extra(&mut bytes, frame, index);
        index += 1;
    }
    bytes
}

fn push_extra(bytes: &mut Vec<u8>, frame: &Frame, index: usize) {
    bytes.push(frame.blend[index]);
    bytes.push(frame.previous[index]);
    bytes.push(frame.sprite_luma[index]);
    bytes.push(0);
}

fn target_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("eightbit-emu-target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn encode(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    group: &wgpu::BindGroup,
    view: &wgpu::TextureView,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    draw_pass(&mut encoder, pipeline, group, view);
    queue.submit([encoder.finish()]);
}

fn draw_pass(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    group: &wgpu::BindGroup,
    view: &wgpu::TextureView,
) {
    let attachment = color_attachment(view);
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("eightbit-emu-present"),
        color_attachments: &[Some(attachment)],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, group, &[]);
    pass.draw(0..3, 0..1);
}

fn color_attachment(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
        depth_slice: None,
    }
}

fn copy_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, GpuError> {
    let padded = padded_stride(width);
    let buffer = read_buffer(device, padded, height);
    copy_into(device, queue, texture, &buffer, padded, height);
    map_buffer(device, &buffer)?;
    Ok(unpack_rows(&buffer, width, height, padded)?)
}

fn padded_stride(width: u32) -> u32 {
    let bytes = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    bytes.div_ceil(align) * align
}

fn read_buffer(device: &wgpu::Device, stride: u32, height: u32) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("eightbit-emu-read"),
        size: u64::from(stride) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn copy_into(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    buffer: &wgpu::Buffer,
    stride: u32,
    height: u32,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        copy_dest(buffer, stride, height),
        texture.size(),
    );
    queue.submit([encoder.finish()]);
}

fn copy_dest(buffer: &wgpu::Buffer, stride: u32, height: u32) -> wgpu::TexelCopyBufferInfo<'_> {
    wgpu::TexelCopyBufferInfo {
        buffer,
        layout: wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(stride),
            rows_per_image: Some(height),
        },
    }
}

fn map_buffer(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Result<(), GpuError> {
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(gpu_error)?;
    receiver.recv().map_err(gpu_error)?.map_err(gpu_error)
}

fn unpack_rows(
    buffer: &wgpu::Buffer,
    width: u32,
    height: u32,
    stride: u32,
) -> Result<Vec<u8>, GpuError> {
    let mapped = buffer.slice(..).get_mapped_range().map_err(gpu_error)?;
    let mut pixels = Vec::new();
    let mut row = 0u32;
    while row < height {
        push_row(&mut pixels, &mapped, width, stride, row);
        row += 1;
    }
    Ok(pixels)
}

fn push_row(pixels: &mut Vec<u8>, mapped: &[u8], width: u32, stride: u32, row: u32) {
    let start = (row * stride) as usize;
    let end = start + (width * 4) as usize;
    pixels.extend_from_slice(&mapped[start..end]);
}

pub fn fit_scale(
    frame_w: u32,
    frame_h: u32,
    window_w: u32,
    window_h: u32,
) -> Option<(u32, u32, u32)> {
    let scale = integer_scale(frame_w, frame_h, window_w, window_h)?;
    let view = centered_viewport(frame_w, frame_h, window_w, window_h, scale);
    Some((scale, view.x, view.y))
}
