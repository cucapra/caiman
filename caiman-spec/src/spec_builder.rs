use crate::spec;
use std::default::Default;

#[derive(Default, Debug)]
pub struct SpecBuilder {
    operations: Vec<spec::Operation>,
}

impl SpecBuilder {
    fn new() -> Self {
        Self { operations: vec![] }
    }

    fn add_operation(&mut self, operation: spec::Operation) {
        self.operations.push(operation);
    }

    fn build(mut self) -> spec::Spec {
        spec::Spec {
            operations: self.operations,
        }
    }
}
