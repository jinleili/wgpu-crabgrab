use crate::wgpu_canvas::WgpuCanvas;
use android_logger::Config;
use app_surface::AppSurface;
use jni::objects::JClass;
use jni::sys::{jint, jlong, jobject};
use jni::JNIEnv;
use jni_fn::jni_fn;
use log::{info, LevelFilter};

#[no_mangle]
#[jni_fn("name.jinleili.wgpu.RustBridge")]
pub fn createWgpuCanvas(env: *mut JNIEnv, _: JClass, surface: jobject, idx: jint) -> jlong {
    log_panics::init();
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Error));
    let canvas = WgpuCanvas::new(AppSurface::new(env as *mut _, surface));
    info!("WgpuCanvas created!");
    Box::into_raw(Box::new(canvas)) as jlong
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu.RustBridge")]
pub fn enterFrame(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let obj = unsafe { &mut *(obj as *mut WgpuCanvas) };
    obj.enter_frame();
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu.RustBridge")]
pub fn dropWgpuCanvas(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let _obj: Box<WgpuCanvas> = unsafe { Box::from_raw(obj as *mut _) };
}
