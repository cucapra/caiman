use super::*;

impl<'context> Context<'context> {
    pub fn new(program: &'context mut ast::Program) -> Context<'context> {
        Context {
            program,
            location: RemoteNodeId::new(),
            value_explication_data: HashMap::new(),
            schedule_explication_data: HashMap::new(),
            meta_data: MetaData::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.initialize_allocations();
    }

    fn initialize_allocations(&mut self) {

    }
}