use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable,Serialize,Deserialize)]
pub struct Instance{ //28 bytes
    pub position: [f32;2],
    pub rotation: f32,
    pub scale: f32,
    pub color: [f32; 3],
}
impl Instance {
    pub fn new(position: [f32;2],color: [f32; 3],rotation:f32,scale: f32)->Self{
        Self{
            position,
            color,
            rotation,
            scale,
        }
    }
    pub fn set_hsl(&mut self, hue: f32,sat: f32){
        let kr = (5.+hue*6.)%6.;
        let kg = (3.+hue*6.)%6.;
        let kb = (1.+hue*6.)%6.;

        let r = 1. - kr.min(4.-kr).clamp(0.,1.0);
        let g = 1. - kr.min(4.-kg).clamp(0.,1.0);
        let b = 1. - kr.min(4.-kb).clamp(0.,1.0);

        self.color = [r,g,b];
    }
}