#[derive(Default)]
pub struct IdGenerator {
    next_id: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn generate(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
