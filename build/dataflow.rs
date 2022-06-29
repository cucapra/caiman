use caiman_spec::spec;
use std::fs::File;
use std::io::Write;

use spec::OperationInputKind as OIK;
/// Return the dataflow type name corresponding to the input
fn dataflow_type_name(input: &spec::OperationInput) -> String {
    let element_type = match input.kind {
        OIK::Type => "ir::TypeId",
        OIK::ImmediateI64 => "i64",
        OIK::ImmediateU64 => "u64",
        OIK::Index => "usize",
        OIK::ExternalCpuFunction => "ir::ExternalCpuFunctionId",
        OIK::ExternalGpuFunction => "ir::ExternalGpuFunctionId",
        OIK::ValueFunction => "ir::ValueFunctionId",
        OIK::Operation => "NodeIndex",
        OIK::Place => "ir::Place",
    };
    if input.is_array {
        format!("Box<[{element_type}]>")
    } else {
        element_type.to_string()
    }
}
macro_rules! write_scope {
    ( $o:expr, {$($pre:tt)*} $b:block) => {
        write_scope!($o, {$($pre)*} $b {""})
    };
    ( $o:expr, {$($pre:tt)*} $b:block {$($post:tt)*}) => {
        $o.write_fmt(format_args!($($pre)*));
        write!($o, "{{\n")?;
        $b
        write!($o, "}}")?;
        $o.write_fmt(format_args!($($post)*));
        write!($o, "\n")?;
    };
}
fn write_formatted_file(
    path: &str,
    contents: impl FnOnce(&mut File) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut o = File::create(path)?;
    contents(&mut o);
    let _ = std::process::Command::new("rustfmt")
        .args(&["--edition", "2021", path])
        .spawn();
    Ok(())
}

pub fn write_base(path: &str, spec: &spec::Spec) -> std::io::Result<()> {
    write_formatted_file(path, |o| {
        write_scope!( o, {"#[derive(Debug)] pub enum Operation"} {
            for operation in spec.operations.iter() {
                write_scope!(o, {"{}", operation.name} {
                    for input in operation.inputs.iter() {
                        write!(o, "{}: {},\n", input.name, dataflow_type_name(&input))?;
                    }
                } {","});
            }
        });
        write_scope!(o, {"impl Operation"} {
            write_scope!(o, {"pub fn for_each_dependency(&self, mut f: impl FnMut(&NodeIndex))"} {
                write_scope!(o, {"match self"} {
                    for operation in spec.operations.iter() {
                        write_scope!(o, {"Self::{}", operation.name} {
                            for input in operation.inputs.iter() {
                                if input.kind == OIK::Operation {
                                    write!(o, "{},\n", input.name)?;
                                }
                            }
                            write!(o, "..\n")?;
                        } {"=>"});
                        write_scope!(o, {""} {
                            for input in operation.inputs.iter() {
                                if input.kind == OIK::Operation {
                                    if input.is_array {
                                        write!(o, "{}.iter().for_each(&mut f);\n", input.name)?;
                                    } else {
                                        write!(o, "f({});\n", input.name)?;
                                    }
                                }
                            }
                        });
                    }
                });
            });
        });
        Ok(())
    })
}

pub fn write_conversion(
    path: &str,
    spec: &spec::Spec,
    src_type: &str,
    dst_type: &str,
) -> std::io::Result<()> {
    write_formatted_file(path, |o| {
        write_scope!(o, {"impl<'a> Convert<{dst_type}, Context<'a>> for {src_type}"} {
            write_scope!(o, {"fn convert(self, context: &Context<'a>) -> Result<{dst_type}, <Context<'a> as ConversionContext>::Error>"} {
                write_scope!(o, {"Ok(match self"} {
                    for operation in spec.operations.iter() {
                        write_scope!(o, {"{src_type}::{}", operation.name} {
                            for input in operation.inputs.iter() {
                                write!(o, "{},\n", input.name)?;
                            }
                        } {"=>"});
                        write_scope!(o, {"{dst_type}::{}", operation.name} {
                            for input in operation.inputs.iter() {
                                write!(o, "{0}: {0}.convert(context)?,\n", input.name)?;
                            }
                        } {","});
                    }
                } {")"});
            });
        });
        Ok(())
    })
}
