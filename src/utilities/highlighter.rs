use std::ops::Index;
use crate::environment::animal::Animals;
use crate::rendering::instance::Instance;

#[derive(Default)]
pub struct Highlighter{
    highlights: Vec<Instance>,
    animals: Vec<usize>,
    pub speed: Condition,
    pub size: Condition,
    pub diet: Condition,
    pub species_id:usize,
    pub species_id_on:bool,
    pub selected_highlight: SelectedHighlight,
}
#[derive(Debug,PartialEq)]
pub enum SelectedHighlight{
    Diet,
    Speed,
    Size,
    Species,
    None,
}

impl Default for SelectedHighlight{
    fn default() -> Self {
        Self::None
    }
}
#[derive(Default)]
pub struct Condition{
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

        match self.selected_highlight {
            SelectedHighlight::Diet =>{
                animals.animals.iter().enumerate().for_each(|(i, animal)| {
                    if !self.diet.bounded || (self.diet.bounded && animal.combat_stats.carnivore_factor <= self.diet.upper && animal.combat_stats.carnivore_factor >= self.diet.lower) {
                        self.highlights.push(Instance {
                            position: [0., 0.],
                            rotation: 0.0,
                            scale: 1.0,
                            color: [animal.combat_stats.carnivore_factor,1.-animal.combat_stats.carnivore_factor,0.1],
                        });
                        self.animals.push(i);
                    }
                });
            },
            SelectedHighlight::Size =>{
                animals.animals.iter().enumerate().for_each(|(i, animal)| {
                    if !self.size.bounded || (self.size.bounded && (animal.body.scale - 0.08)/0.42 <= self.size.upper && (animal.body.scale - 0.08)/0.42 >= self.size.lower) {
                        self.highlights.push(Instance {
                            position: [0., 0.],
                            rotation: 0.0,
                            scale: 1.0,
                            color: [1.0,1.0 - (animal.body.scale - 0.08)/0.42 * 0.7,0.1],
                        });
                        self.animals.push(i);
                    }
                });
            }
            SelectedHighlight::Speed =>{
                animals.animals.iter().enumerate().for_each(|(i, animal)| {
                    if !self.speed.bounded || (self.speed.bounded && (animal.combat_stats.speed-0.5)/3.5 <= self.speed.upper && (animal.combat_stats.speed-0.5)/3.5 >= self.speed.lower) {
                        self.highlights.push(Instance {
                            position: [0., 0.],
                            rotation: 0.0,
                            scale: 1.0,
                            color: [(animal.combat_stats.speed-0.5) * 2./119.,(animal.combat_stats.speed-0.5) * 10./357.,1.],
                        });
                        self.animals.push(i);
                    }
                });
            },
            SelectedHighlight::Species =>{
                animals.animals.iter().enumerate().for_each(|(i, animal)| {
                    if animal.species_id == self.species_id {
                        self.highlights.push(Instance {
                            position: [0., 0.],
                            rotation: 0.0,
                            scale: 1.0,
                            color: [1.,1.,1.],
                        });
                        self.animals.push(i);
                    }
                });
            }
            SelectedHighlight::None=>{},
        }
    }
    pub fn instances(&self) -> &Vec<Instance>{&self.highlights}
    pub fn count(&self)->usize{
        self.highlights.len()
    }
}