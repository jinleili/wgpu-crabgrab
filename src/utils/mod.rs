pub mod load_texture;
pub use load_texture::AnyTexture;
pub mod node;

mod buffer;
pub use buffer::BufferObj;

use bytemuck::{Pod, Zeroable};

pub static DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MVPMatUniform {
    pub mvp: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SceneUniform {
    pub mvp: [[f32; 4]; 4],
    pub viewport_pixels: [f32; 2],
    pub padding: [f32; 2],
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn application_root_dir() -> String {
    let location = web_sys::window().unwrap().location();
    let host = location.host().unwrap();
    if host.contains("localhost") || host.contains("127.0.0.1") {
        String::from("http://") + &host + "/"
    } else {
        if host.contains("jinleili.github.io") {
            location.href().unwrap()
        } else {
            String::from("https://cannot.access/")
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn application_root_dir() -> String {
    use std::env;
    use std::fs;

    match env::var("PROFILE") {
        Ok(_) => String::from(env!("CARGO_MANIFEST_DIR")),
        Err(_) => {
            let mut path = env::current_exe().expect("Failed to find executable path.");
            while let Ok(target) = fs::read_link(path.clone()) {
                path = target;
            }
            if cfg!(any(
                target_os = "macos",
                target_os = "windows",
                target_os = "linux"
            )) {
                path = path.join("../../../assets/").canonicalize().unwrap();
            }

            String::from(path.to_str().unwrap())
        }
    }
}

use std::path::PathBuf;
#[allow(unused)]
pub(crate) fn get_texture_file_path(name: &str) -> PathBuf {
    PathBuf::from(application_root_dir()).join(name)
}
