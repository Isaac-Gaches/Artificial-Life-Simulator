pub struct Stats{
    pub fps: Vec<[f64;2]>,
    step: usize,
}
impl Stats{
    pub fn update(&mut self,framerate: f64){
        self.fps.push([self.step as f64,framerate]);
        self.step+=1;
    }
}
