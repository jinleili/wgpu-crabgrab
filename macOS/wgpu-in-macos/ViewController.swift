//
//  ViewController.swift
//  wgpu-in-macos
//
//  Created by 李金磊 on 2024/7/13.
//

import Cocoa
import QuartzCore

class ViewController: NSViewController {
    @IBOutlet var metalV: MetalView!
    var wgpuCanvas: OpaquePointer?
    
     var displayLink: CADisplayLink?
    
    override func viewDidLoad() {
        super.viewDidLoad()
    }
    
    override func viewDidAppear() {
        super.viewDidAppear()
        if displayLink == nil {
            displayLink = self.view.window!.screen!.displayLink(target: self, selector: #selector(enterFrame))
            
            displayLink?.add(to: .current, forMode: .default)
            displayLink?.isPaused = true
        }
        
        if wgpuCanvas == nil {
            let viewPointer = Unmanaged.passUnretained(self.metalV).toOpaque()
            let metalLayer = Unmanaged.passUnretained(self.metalV.layer!).toOpaque()
            
            let viewObj = ios_view_obj(view: viewPointer, metal_layer: metalLayer,maximum_frames: Int32(60), callback_to_swift: callback_to_swift)
            
            wgpuCanvas = create_wgpu_canvas(viewObj)
            
            self.view.window?.delegate = self
        }
        self.displayLink?.isPaused = false
       
    }
    
    override func viewWillDisappear() {
        super.viewWillDisappear()
        displayLink?.isPaused = true
    }
    
    @objc func enterFrame() {
        guard let canvas = self.wgpuCanvas else {
            return
        }
        // call rust
        enter_frame(canvas)
    }

}

extension ViewController: NSWindowDelegate {
    
    func windowDidResize(_ notification: Notification) {
        // 在这里处理窗口大小变化后的逻辑
        let newSize = self.view.window?.frame.size
        print("Window resized, new size: \(String(describing: newSize))")
        guard let canvas = self.wgpuCanvas else {
            return
        }
        resize(canvas);
    }
}

func callback_to_swift(arg: Int32) {
    DispatchQueue.main.async {
        switch arg {
        case 0:
            print("wgpu canvas created!")
            break
        case 1:
            print("canvas enter frame")
            break
            
        default:
            break
        }
    }
    
}

