mod wgpu_canvas;
use std::sync::Arc;

pub use wgpu_canvas::WgpuCanvas;

#[cfg_attr(target_os = "ios", path = "ffi/ios.rs")]
#[cfg_attr(target_os = "macos", path = "ffi/ios.rs")]
mod ffi;

#[cfg(all(target_os = "macos", target_os = "ios"))]
pub use ffi::*;

mod capture;

mod utils;
use utils::*;

#[derive(Clone)]
pub struct DeviceQueue {
    pub config: wgpu::SurfaceConfiguration,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}
