//
//  MetalView.swift
//
//  Created by LiJinlei on 2018/11/23.
//

import AppKit
import Foundation

class MetalView: NSView {
    var contentScaleFactor: CGFloat {
      get {
          // 返回合适的缩放比例，这里假设获取屏幕的 backingScaleFactor
          return NSScreen.main?.backingScaleFactor ?? 1.0
      }
    }
    override init(frame frameRect: NSRect) {
       super.init(frame: frameRect)
       self.wantsLayer = true
       self.layer = CAMetalLayer()
       setupMetalLayer()
   }

   required init?(coder: NSCoder) {
       super.init(coder: coder)
       self.wantsLayer = true
       self.layer = CAMetalLayer()
       setupMetalLayer()
   }
   
    private func setupMetalLayer() {
        guard let metalLayer = self.layer as? CAMetalLayer else {
            fatalError("Expected layer to be a CAMetalLayer")
        }
        
        metalLayer.device = MTLCreateSystemDefaultDevice()
        metalLayer.pixelFormat = .bgra8Unorm
        metalLayer.contentsScale = NSScreen.main?.backingScaleFactor ?? 1.0
        // https://developer.apple.com/documentation/quartzcore/cametallayer/1478157-presentswithtransaction/
        metalLayer.presentsWithTransaction = false
        metalLayer.framebufferOnly = true
    }
}

