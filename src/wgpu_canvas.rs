use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use crate::{
    capture, node::BufferlessFullscreenNode, utils, BufferObj, DeviceQueue, MVPMatUniform,
};
use app_surface::{AppSurface, SurfaceFrame};

pub struct WgpuCanvas {
    pub app_surface: AppSurface,
    show_capture: Arc<Mutex<BufferlessFullscreenNode>>,
}

#[allow(dead_code)]
impl WgpuCanvas {
    pub fn new(app_surface: AppSurface) -> Self {
        let shader = app_surface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../wgsl_shader/bufferless.wgsl").into(),
                ),
            });
        let dq = DeviceQueue {
            config: app_surface.config.clone(),
            device: app_surface.device.clone(),
            queue: app_surface.queue.clone(),
        };
        let show_capture = Arc::new(Mutex::new(BufferlessFullscreenNode::new(
            dq.clone(),
            app_surface.config.format,
            &shader,
            None,
            1,
        )));
        let display_node = show_capture.clone();

        let _capture_handle = std::thread::spawn(|| capture::run(dq, display_node));

        let instance = WgpuCanvas {
            app_surface,
            show_capture: show_capture,
        };

        if let Some(callback) = instance.app_surface.callback_to_app {
            callback(0);
        }
        instance
    }

    pub fn enter_frame(&mut self) {
        let device = &self.app_surface.device;
        let queue = &self.app_surface.queue;
        let (frame, view) = self.app_surface.get_current_frame_view(None);
        {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let show_capture = self.show_capture.lock().unwrap();
            show_capture.draw(
                &view,
                &mut encoder,
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.,
                }),
            );

            queue.submit(Some(encoder.finish()));
        }
        frame.present();

        if let Some(_callback) = self.app_surface.callback_to_app {
            // callback(1);
        }
    }

    pub fn resize(&mut self) {
        self.app_surface.resize_surface();
        let mut show_capture = self.show_capture.lock().unwrap();
        show_capture.resize(self.app_surface.config.clone());
    }
}
