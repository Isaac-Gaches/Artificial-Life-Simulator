pub struct SimParams{
    pub steps_per_frame: i32,
    pub highlighted_species: i32,
}
impl Default for SimParams{
    fn default() -> Self {
        Self{
            steps_per_frame: 1,
            highlighted_species: -1,
        }
    }
}