use rand::Rng;
#[derive(Clone)]
pub struct Network{
    layers: Vec<Layer>,
}
#[derive(Clone)]
struct Layer{
    neurons: Vec<Neuron>,
}
#[derive(Clone)]
struct Neuron{
    bias: f32,
    weights: Vec<f32>,
}
impl Network{
    pub fn propagate(&self,inputs: Vec<f32>) -> Vec<f32>{
        self.layers.iter().fold(inputs,|inputs,layer| layer.propagate(inputs))
    }
    pub fn random(layers: &[usize]) -> Self{
        let layers = layers.windows(2).map(|layers| { Layer::random(layers[0], layers[1]) }).collect();
        Self { layers }
    }
    pub fn mutate(&mut self){
        self.layers.iter_mut().for_each(|layer| layer.mutate());
    }
}
impl Layer{
    fn propagate(&self,inputs: Vec<f32>) -> Vec<f32>{
        self.neurons.iter().map(|neuron| neuron.propagate(&inputs)).collect()
    }
    fn random(input_size: usize, output_size: usize) -> Self{
        let neurons = (0..output_size).map(|_| Neuron::random(input_size)).collect();
        Self { neurons }
    }
    fn mutate(&mut self){
        self.neurons.iter_mut().for_each(|neuron| neuron.mutate());
    }
}
impl Neuron{
    fn propagate(&self,inputs: &[f32]) -> f32{
        let output = inputs.iter().zip(&self.weights).map(|(input, weight)| input * weight).sum::<f32>();
        (self.bias + output).max(0.0)
    }
    fn random(input_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let bias = rng.gen_range(-1.0..=1.0);
        let weights = (0..input_size).map(|_| rng.gen_range(-1.0..=1.0)).collect();
        Self { bias, weights }
    }
    fn mutate(&mut self){
        let mut rng = rand::thread_rng();
        self.weights.iter_mut().for_each(|weight| if rng.gen_bool(0.1){
            *weight += rng.gen_range(-0.4..0.4);
        });
        if rng.gen_bool(0.1){
            self.bias += rng.gen_range(-0.4..0.4);
        }
    }
}