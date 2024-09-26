use crate::rendering::render::Instance;

pub struct RockMap{
    rocks: Vec<u8>
}
impl RockMap{
    pub fn new()->Self{
        Self{
            rocks: vec![1],
        }
    }
    pub fn instances(&self)->Vec<Instance>{
        self.rocks.iter().map(|rock|{
            Instance::new([0.,0.], [0.1,0.1,0.1], 0.0, 0.0)
        }).collect()
    }
}