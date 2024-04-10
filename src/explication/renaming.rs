use crate::assembly::ast;

fn give_names(&mut program: ast::Program) -> MetaData {
    for declaration in self.program.declarations.iter_mut() {
        match declaration {
            // add phi nodes
            ast::Declaration::Funclet(f) => {
                let mut index = 0;
                for arg in &f.header.args {
                    f.commands.insert(
                        index,
                        ast::NamedCommand {
                            name: arg.name.clone(),
                            command: ast::Command::Node(ast::Node::Phi { index: Some(index) }),
                        },
                    );
                    index += 1;
                }
                index = 0;
                for command in f.commands.iter_mut() {
                    // give names to unnamed things (even tail edges, just in case)
                    command.name = match &command.name {
                        None => Some(NodeId(format!("~{}", index))),
                        n => n.clone(),
                    };
                    index += 1;
                }
            }
            _ => {}
        }
    }
}