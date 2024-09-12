use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone,Serialize,Deserialize)]
pub struct Network{
    pub layers: Vec<Layer>,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Layer{
    pub neurons: Vec<Neuron>,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Neuron{
    pub activation: f32,
    pub bias: f32,
    pub weights: Vec<f32>,
}
impl Network{
    pub fn input(&mut self,inputs: Vec<f32>){
        self.layers[0].neurons.iter_mut().zip(inputs).for_each(|(neuron,input)|{
            neuron.activation = input;
        });
    }
    pub fn propagate(&mut self) -> Vec<f32>{
        let mut inputs = self.layers[0].activations();

        for i in 1..self.layers.len(){
            inputs = self.layers[i].propagate(inputs);
        }

        inputs
    }
    pub fn random(layers: &[usize]) -> Self{
        let mut layers = layers.to_vec();
        layers.insert(0,0);
        let layers = layers.windows(2).map(|layers| { Layer::random(layers[0], layers[1]) }).collect();
        Self { layers }
    }
    pub fn mutate(&mut self,strength: f32){
        for i in 1..self.layers.len(){
            self.layers[i].mutate(strength);
        }
    }
    pub fn compare(&self, other: &Network) -> f32{
        self.layers.iter().zip(other.layers.iter()).map(|layer| { layer.0.compare(layer.1) } ).sum::<f32>()
    }
}
impl Layer{
    fn activations(&self)->Vec<f32>{
        self.neurons.iter().map(|neuron| neuron.activation).collect()
    }
    fn propagate(&mut self,inputs: Vec<f32>) -> Vec<f32>{
        self.neurons.iter_mut().map(|neuron| neuron.propagate(&inputs)).collect()
    }
    fn random(input_size: usize, output_size: usize) -> Self{
        let neurons = (0..output_size).map(|_| Neuron::random(input_size)).collect();
        Self { neurons }
    }
    fn mutate(&mut self,strength: f32){
        self.neurons.iter_mut().for_each(|neuron| neuron.mutate(strength));
    }
    fn compare(&self, other: &Layer) -> f32{
        self.neurons.iter().zip(other.neurons.iter()).map(|neuron|{ neuron.0.compare(neuron.1) }).sum::<f32>()
    }
}
impl Neuron{
    fn propagate(&mut self,inputs: &[f32]) -> f32{
        self.activation = 0.;

        let input = inputs.iter().zip(&self.weights).map(|(input, weight)| input * weight).sum::<f32>();
        self.activation = (input+self.bias).tanh();

        self.activation
    }
    fn random(input_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let decay = rng.gen_range(-1.0..=1.0);
        let weights = (0..input_size).map(|_| rng.gen_range(-1.0..=1.0)).collect();
        Self { activation: 0.0, weights, bias: decay }
    }
    fn mutate(&mut self,strength: f32){
        let mut rng = rand::thread_rng();
        self.weights.iter_mut().for_each(|weight| if rng.gen_bool(strength as f64){
            *weight += rng.gen_range(-strength*0.1..=strength * 0.1);
        });
        if rng.gen_bool(strength as f64){
            self.bias += rng.gen_range(-strength*0.1..=strength * 0.1);
        }
    }
    fn compare(&self, other: &Neuron) -> f32{
        self.weights.iter().zip(other.weights.iter()).map(|weight| { (weight.0 - weight.1).abs() }).sum::<f32>() + (self.bias -other.bias).abs()
    }
}