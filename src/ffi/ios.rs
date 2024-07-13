use crate::wgpu_canvas::WgpuCanvas;
use app_surface::{AppSurface, IOSViewObj};

#[no_mangle]
pub fn create_wgpu_canvas(ios_obj: IOSViewObj) -> *mut libc::c_void {
    println!(
        "create_wgpu_canvas, maximum frames: {}",
        ios_obj.maximum_frames
    );
    let obj = WgpuCanvas::new(AppSurface::new(ios_obj));
    // 使用 Box 对 Rust 对象进行装箱操作。
    // 我们无法将 Rust 对象直接传递给外部语言，通过装箱来传递此对象的胖指针
    let box_obj = Box::new(obj);
    // into_raw 返回指针的同时，将此对象的内存管理权转交给调用方
    Box::into_raw(box_obj) as *mut libc::c_void
}

#[no_mangle]
pub fn enter_frame(obj: *mut libc::c_void) {
    // 获取到指针指代的 Rust 对象的可变借用
    let obj = unsafe { &mut *(obj as *mut WgpuCanvas) };
    obj.enter_frame();
}

#[no_mangle]
pub fn resize(obj: *mut libc::c_void) {
    // 获取到指针指代的 Rust 对象的可变借用
    let obj = unsafe { &mut *(obj as *mut WgpuCanvas) };
    obj.resize();
}