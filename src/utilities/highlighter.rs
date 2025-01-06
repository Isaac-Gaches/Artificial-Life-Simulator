use std::ops::Index;
use crate::environment::animal::Animals;
use crate::rendering::instance::Instance;

#[derive(Default)]
pub struct Highlighter{
    highlights: Vec<Instance>,
    animals: Vec<usize>,
    pub speed: Condition,
}
#[derive(Default)]
pub struct Condition{
    pub on: bool,
    pub bounded: bool,
    pub lower: f32,
    pub upper: f32,
}

impl Highlighter{
    pub fn move_highlights(&mut self, animals: &Animals){
        self.highlights.iter_mut().zip(self.animals.iter()).for_each(|(highlight,id)|{
            highlight.position = animals.animals.index(*id).body.position;
        });
    }
    pub fn set_highlights(&mut self, animals: &Animals){
        self.highlights.clear();
        self.animals.clear();
        if self.speed.on {
            animals.animals.iter().enumerate().for_each(|(i, animal)| {
                if self.speed.on && (!self.speed.bounded || (self.speed.bounded && (animal.combat_stats.speed-0.5)/3.5 <= self.speed.upper && (animal.combat_stats.speed-0.5)/3.5 >= self.speed.lower)) {
                    self.highlights.push(Instance {
                        position: [0., 0.],
                        rotation: 0.0,
                        scale: 1.0,
                        color: [(animal.combat_stats.speed-0.5) * 2./119.,(animal.combat_stats.speed-0.5) * 10./357.,1.],
                    });
                    self.animals.push(i);
                }
            });
        }
    }
    pub fn instances(&self) -> &Vec<Instance>{&self.highlights}
    pub fn count(&self)->usize{
        self.highlights.len()
    }
}