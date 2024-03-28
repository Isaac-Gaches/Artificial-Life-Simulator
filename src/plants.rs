use crate::animal::{Animal, AnimalBody};

pub struct Plant{
    body: PlantBody,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PlantBody {
    position: [f32; 2],
    color: [f32; 3],
    pad:[u32;3], //align to 16 bytes
}

pub struct Plants{
    plants: Vec<Plant>,
}
impl Plant{
    pub fn body(&self)-> PlantBody {
        self.body.clone()
    }
    pub fn new_test(x: f32, y: f32)->Self{
        Self{
            body: PlantBody {
                position: [(x*0.01)-9., (y*0.01)-0.97],
                color: [1.0-x*0.001, y*0.01, y*0.01],
                pad: [0,0,0],
            }
        }
    }
}
impl Plants{
    pub fn genesis()->Self{
        let plants = (0..200000).map(|i| {
            let x = i as f32 % 1000.;
            let y = (i as f32/1000.).trunc();
            Plant::new_test(x,y)

        }).collect::<Vec<Plant>>();

        Self{
            plants
        }
    }
    pub fn bodies(&self) -> Vec<PlantBody>{
        self.plants.iter().map(|animal| animal.body() ).collect::<Vec<PlantBody>>()
    }
    pub fn count(&self)->usize{
        self.plants.len()
    }
}