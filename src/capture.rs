use crabgrab::{feature::bitmap::VideoFrameBitmap as _, prelude::*};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{node::BufferlessFullscreenNode, DeviceQueue};

pub fn run(dq: DeviceQueue, display_node: Arc<Mutex<BufferlessFullscreenNode>>) {
    let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    let future = runtime.spawn(async {
        let token = match CaptureStream::test_access(false) {
            Some(token) => token,
            None => CaptureStream::request_access(false).await.expect("Expected capture access")
        };
        let filter = CapturableContentFilter::NORMAL_WINDOWS;
        let content = CapturableContent::new(filter).await.unwrap();
        let window = content.windows().filter(|window| {
            let app_identifier = window.application().identifier();
            window.title().len() != 0 && app_identifier.to_lowercase().contains("chrome")
        }).next();
        match window {
            Some(window) => {
                println!("capturing window: {}", window.title()); 
                let config = CaptureConfig::with_window(window, CapturePixelFormat::Bgra8888)
                .unwrap();
                // .with_wgpu_device(gfx.clone())
                // .unwrap();
                let mut stream = CaptureStream::new(token, config, move |stream_event| {
                    match stream_event {
                        Ok(event) => {
                            match event {
                                StreamEvent::Video(frame) => {
                                    println!("Got frame: {}", frame.frame_id());
                                    match frame.get_bitmap() {
                                        Ok(bitmap) => {
                                            match bitmap {
                                                crabgrab::feature::bitmap::FrameBitmap::BgraUnorm8x4(data) => {
                                                    let mut display_node = display_node.lock().unwrap();
                                                    display_node.update(wgpu::TextureFormat::Bgra8UnormSrgb, data);
                                                },
                                                crabgrab::feature::bitmap::FrameBitmap::ArgbUnormPacked2101010(_) => println!("format: ArgbUnormPacked2101010"),
                                                crabgrab::feature::bitmap::FrameBitmap::RgbaF16x4(_) => println!("format: RgbaF16x4"),
                                                crabgrab::feature::bitmap::FrameBitmap::YCbCr(_) => println!("format: YCbCr"),
                                            }
                                        },
                                        Err(e) => {
                                            println!("Bitmap error: {:?}", e);
                                        }
                                    }
                                },
                                _ => {}
                            }
                        },
                        Err(error) => {
                            println!("Stream error: {:?}", error);
                        }
                    }
                }).unwrap();
                println!("stream created!"); 
                            tokio::task::block_in_place(|| std::thread::sleep(Duration::from_millis(3600000)));
                stream.stop().unwrap();
            },
            None => { println!("Failed to find window"); }
        }
    });
    runtime.block_on(future).unwrap();
    runtime.shutdown_timeout(Duration::from_millis(100000));
}
