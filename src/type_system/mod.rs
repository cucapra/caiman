pub mod scheduling;
pub mod spec_checker;
#[macro_use]
pub mod error;

pub fn check_program(program: & super::ir::Program) -> Result<(), error::Error> {
    for (funclet_id, funclet) in program.funclets.iter() {
        let funclet_error_contextualizer = |writer : &mut dyn std::fmt::Write| {write!(writer, "In funclet #{}", funclet_id)};
        let funclet_error_context = error::ErrorContext::new(None, Some(& funclet_error_contextualizer ));

		if funclet.kind != super::ir::FuncletKind::ScheduleExplicit {
			continue;
		}

        let mut funclet_checker = scheduling::FuncletChecker::new(&program, funclet);

        for (current_node_id, node) in funclet.nodes.iter().enumerate() {
            let node_error_contextualizer = |writer : &mut dyn std::fmt::Write| { write!(writer, "While compiling node #{}: {:?}", current_node_id, node) };
            let node_error_context = error::ErrorContext::new(Some(& funclet_error_context), Some(& node_error_contextualizer ));

            funclet_checker.check_next_node(&node_error_context, current_node_id)?;
		}

        let tail_error_contextualizer = |writer : &mut dyn std::fmt::Write| { write!(writer, "While compiling tail edge: {:?}", funclet.tail_edge) };
        let tail_error_context = error::ErrorContext::new(Some(& funclet_error_context), Some(& tail_error_contextualizer ));
        funclet_checker.check_tail_edge(& tail_error_context)?;
	}
	return Ok(());
}