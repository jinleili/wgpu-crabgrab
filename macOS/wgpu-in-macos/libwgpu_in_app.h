//
//  libwgpu_on_ios.h
//
//  Created by LiJinlei on 2021/9/10.
//

#ifndef libwgpu_in_app_h
#define libwgpu_in_app_h

#include <stdint.h>

// 这个不透明结构体用来指代 Rust 端的 WgpuCanvas 对象
struct wgpu_canvas;

struct ios_view_obj
{
    void *view;
    // CAMetalLayer
    void *metal_layer;
    int maximum_frames;
    void (*callback_to_swift)(int32_t arg);
};

struct wgpu_canvas *create_wgpu_canvas(struct ios_view_obj object);
void enter_frame(struct wgpu_canvas *data);
void resize(struct wgpu_canvas *data);

#endif /* libwgpu_in_app_h */
