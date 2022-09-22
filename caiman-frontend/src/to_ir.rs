use caiman::ir;
use crate::ast;
use crate::error;

pub enum ToIRError
{
}

//type Index<T> = HashMap<T, usize>;

pub fn go(
    filename: &str,
    _ast: &ast::ParsedProgram,
) -> Result<ir::Program, error::Error>
{
    let pipe1 = ir::Pipeline {
        name: String::from("A"),
        entry_funclet: 0,
        yield_points: Default::default(),
    };
    Ok(
        ir::Program {
            pipelines: vec![pipe1],
            ..
            Default::default()
        }
    )
}

//fn make_funclet(




