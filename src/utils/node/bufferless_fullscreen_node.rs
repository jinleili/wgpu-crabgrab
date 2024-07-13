use crate::{load_texture, AnyTexture, BufferObj, DeviceQueue};
use crabgrab::prelude::*;
use wgpu::{PrimitiveTopology, ShaderModule, TextureFormat};

use super::BindGroupData;

pub struct BufferlessFullscreenNode {
    dq: DeviceQueue,
    tex_width: usize,
    tex_height: usize,
    tex: Option<AnyTexture>,
    uniform: Option<BufferObj>,
    bind_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

#[allow(dead_code)]
impl BufferlessFullscreenNode {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dq: DeviceQueue,
        target_format: TextureFormat,
        shader_module: &ShaderModule,
        color_blend_state: Option<wgpu::BlendState>,
        sample_count: u32,
    ) -> Self {
        let pipeline_vertex_buffers = [];
        let blend_state = if color_blend_state.is_some() {
            color_blend_state
        } else {
            Some(wgpu::BlendState::ALPHA_BLENDING)
        };
        let pipeline = dq
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("bufferless fullscreen pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: shader_module,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &pipeline_vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader_module,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: blend_state,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                // the bufferless vertices are in clock-wise order
                primitive: wgpu::PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Front),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: sample_count,
                    ..Default::default()
                },
                multiview: None,
            });

        Self {
            dq,
            tex_height: 0,
            tex_width: 0,
            tex: None,
            uniform: None,
            bind_group: None,
            pipeline,
        }
    }

    pub fn update(
        &mut self,
        format: wgpu::TextureFormat,
        data: FrameBitmapBgraUnorm8x4<Box<[[u8; 4]]>>,
    ) {
        let size = wgpu::Extent3d {
            width: data.width as u32,
            height: data.height as u32,
            depth_or_array_layers: 1,
        };
        if data.width != self.tex_width || data.height != self.tex_height {
            self.tex_width = data.width;
            self.tex_height = data.height;
            let factor: (f32, f32) = self.calc_fit();

            println!("纹理尺寸发生变化，需重建 bind group, factor: {:?}", factor);

            let unifor_buf = BufferObj::create_uniform_buffer(
                &self.dq.device,
                &[1. / factor.0, 1. / factor.1],
                None,
            );

            let tex = load_texture::empty(
                &self.dq.device,
                format,
                size,
                None,
                wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                None,
            );
            self.tex = Some(tex);

            let sampler = self.dq.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            // 准备绑定组需要的数据
            let bind_group_data = BindGroupData {
                uniforms: vec![&unifor_buf],
                inout_tv: vec![(self.tex.as_ref().unwrap(), None)],
                samplers: vec![&sampler],
                visibilitys: vec![
                    wgpu::ShaderStages::VERTEX,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::ShaderStages::FRAGMENT,
                ],
                ..Default::default()
            };
            self.bind_group = Some(create_bind_group(
                &self.dq.device,
                &bind_group_data,
                &self.pipeline.get_bind_group_layout(0),
            ));
            self.uniform = Some(unifor_buf);
        } else {
            println!(
                "纹理尺寸不变，直接更新上一次的 texture, size: {}, {}",
                data.width, data.height
            );
            // let texture =
            self.dq.queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &self.tex.as_ref().unwrap().tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &data.data.concat(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * size.width),
                    rows_per_image: None,
                },
                size,
            );
        }
    }

    pub fn resize(&mut self, config: wgpu::SurfaceConfiguration) {
        self.dq.config = config;
        let factor: (f32, f32) = self.calc_fit();
        if let Some(uniform) = self.uniform.as_mut() {
            self.dq.queue.write_buffer(
                &uniform.buffer,
                0,
                bytemuck::bytes_of(&[1. / factor.0, 1. / factor.1]),
            )
        }
    }

    pub fn draw(
        &self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) {
        if self.bind_group.is_none() {
            return;
        }
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("bufferless rpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        self.draw_by_pass(&mut rpass);
    }

    pub fn draw_by_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        if let Some(bind_group) = self.bind_group.as_ref() {
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
    }

    fn calc_fit(&mut self) -> (f32, f32) {
        let (from_width, from_height) = (self.tex_width, self.tex_height);
        let (to_width, to_height) = (self.dq.config.width, self.dq.config.height);

        // 计算宽高比
        let from_ratio = from_width as f64 / from_height as f64;
        let to_ratio = to_width as f64 / to_height as f64;

        // 比较宽高比以确定按哪个维度缩放
        let (target_width, target_height) = if from_ratio > to_ratio {
            // 按宽度缩放
            let scaled_width = to_width;
            let scaled_height = (to_width as f64 / from_ratio) as u32;
            (scaled_width, scaled_height)
        } else {
            // 按高度缩放
            let scaled_height = to_height;
            let scaled_width = (to_height as f64 * from_ratio) as u32;
            (scaled_width, scaled_height)
        };
        (
            target_width as f32 / to_width as f32,
            target_height as f32 / to_height as f32,
        )
    }
}

pub fn create_bind_group(
    device: &wgpu::Device,
    bg_data: &BindGroupData,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let mut entries: Vec<wgpu::BindGroupEntry> = vec![];
    let mut b_index = 0_u32;
    for uniform in bg_data.uniforms.iter() {
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: uniform.buffer.as_entire_binding(),
        });
        b_index += 1;
    }

    for storage_buf in bg_data.storage_buffers.iter() {
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: storage_buf.buffer.as_entire_binding(),
        });
        b_index += 1;
    }

    for a_tex in bg_data.inout_tv.iter() {
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: wgpu::BindingResource::TextureView(&a_tex.0.tex_view),
        });
        b_index += 1;
    }

    for sampler in &bg_data.samplers {
        entries.push(wgpu::BindGroupEntry {
            binding: b_index,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
        b_index += 1;
    }

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &entries,
        label: None,
    })
}
