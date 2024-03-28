pub struct Animal{
    body: AnimalBody,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnimalBody {
    position: [f32; 2],
    color: [f32; 3],
    pad:[u32;3], //align to 16 bytes
}
impl Animal{
    pub fn body(&self)-> AnimalBody {
        self.body.clone()
    }
    pub fn new_test(x: f32, y: f32)->Self{
        Self{
            body: AnimalBody {
                position: [(x*0.01)-9., (y*0.01)-0.99],
                color: [x*0.001, 1.0 - y*0.01, y*0.01],
                pad: [0,0,0],
            }
        }
    }
}
pub struct Animals{
    animals: Vec<Animal>
}
impl Animals{
    pub fn genesis()->Self{
        let animals = (0..200000).map(|i| {
            let x = i as f32 % 1000.;
            let y = (i as f32/1000.).trunc();
            Animal::new_test(x,y)

        }).collect::<Vec<Animal>>();

        Self{
            animals
        }
    }
    pub fn bodies(&self) -> Vec<AnimalBody>{
        self.animals.iter().map(|animal| animal.body() ).collect::<Vec<AnimalBody>>()
    }
    pub fn count(&self)->usize{
        self.animals.len()
    }
}