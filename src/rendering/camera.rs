use winit::dpi::{PhysicalSize};
use crate::utilities::input_manager::Inputs;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub position: [f32;2],
    pub zoom: f32,
    pub ratio: f32,
}

impl Camera {
    pub fn update(&mut self,inputs: &Inputs,size: &PhysicalSize<u32>) {
        self.position[1] += if inputs.up {0.01 / self.zoom} else if inputs.down {-0.01 / self.zoom} else {0.0};
        self.position[0] += if inputs.right {0.01 / self.zoom} else if inputs.left {-0.01  / self.zoom} else {0.0};
        self.zoom += if inputs.plus {0.003} else if inputs.minus {-0.003} else {0.0};
        self.zoom = self.zoom.clamp(0.03,0.5);
        self.ratio = size.height as f32/size.width as f32;
    }
}