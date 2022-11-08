pub struct FastCollection<T> {
    pub elements: Vec<T>,
    pub free_map: Vec<usize>,
}
