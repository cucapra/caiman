//! Expands records into their respective fields in the inputs and outputs of
//! function signatures.

use std::collections::BTreeSet;

use crate::{
    error::Info,
    parse::ast::{DataType, FlaggedType, FullType, SpecType, Tag, WGPUFlags, WGPUSettings},
};

/// Pushes the fields of a record onto the end of a list of input or output arguments.
/// Prefixes the field name with `arg_prefix::`.
#[allow(clippy::too_many_arguments)]
fn insert_record_args(
    new_inputs: &mut Vec<(String, Option<FullType>)>,
    record_types: &Vec<(String, DataType)>,
    read: &BTreeSet<String>,
    flags: &BTreeSet<WGPUFlags>,
    settings: &BTreeSet<WGPUSettings>,
    tags: &[Tag],
    info: Info,
    arg_prefix: &str,
    remote: bool,
) {
    for (field_nm, field_typ) in record_types {
        let mut flags = flags.clone();
        if remote {
            if read.contains(field_nm) {
                flags.insert(WGPUFlags::MapRead);
            }
            flags.insert(WGPUFlags::Storage);
            flags.insert(WGPUFlags::CopyDst);
        }
        new_inputs.push((
            format!("{arg_prefix}::{field_nm}"),
            Some(FullType {
                base: Some(FlaggedType {
                    base: field_typ.clone(),
                    flags,
                    info,
                    settings: settings.clone(),
                }),
                tags: tags
                    .iter()
                    .filter(|t| t.quot_var.spec_type == SpecType::Timeline)
                    .cloned()
                    .collect(),
            }),
        ));
    }
}

/// Replaces the input or output types of a function to include all fields of any
/// record crossing the function boundary.
///
/// Renames the fields of the records to be `record_name::field`.
fn replace_io<T: IntoIterator<Item = (String, Option<FullType>)>>(
    input: T,
) -> Vec<(String, Option<FullType>)> {
    let mut new_inputs = Vec::new();
    for (input_nm, input_typ) in input {
        match input_typ {
            Some(FullType {
                base:
                    Some(FlaggedType {
                        base:
                            ref b @ (DataType::Fence(Some(ref t)) | DataType::Encoder(Some(ref t))),
                        flags,
                        info,
                        settings,
                    }),
                tags,
            }) => {
                if let DataType::RemoteObj { all, read } = &**t {
                    new_inputs.push((
                        input_nm.clone(),
                        Some(FullType {
                            base: Some(FlaggedType {
                                base: b.clone(),
                                flags: flags.clone(),
                                info,
                                settings: settings.clone(),
                            }),
                            tags: tags.clone(),
                        }),
                    ));
                    insert_record_args(
                        &mut new_inputs,
                        all,
                        read,
                        &flags,
                        &settings,
                        &tags,
                        info,
                        &input_nm,
                        true,
                    );
                } else {
                    panic!("Unexpected inner type of fence/encoder");
                }
            }
            Some(FullType {
                base:
                    Some(FlaggedType {
                        base: DataType::RemoteObj { all, read },
                        flags,
                        info,
                        settings,
                    }),
                tags,
            }) => insert_record_args(
                &mut new_inputs,
                &all,
                &read,
                &flags,
                &settings,
                &tags,
                info,
                &input_nm,
                true,
            ),
            Some(FullType {
                base:
                    Some(FlaggedType {
                        base: DataType::Record(types),
                        flags,
                        info,
                        settings,
                    }),
                tags,
            }) => insert_record_args(
                &mut new_inputs,
                &types,
                &BTreeSet::new(),
                &flags,
                &settings,
                &tags,
                info,
                &input_nm,
                false,
            ),
            _ => {
                new_inputs.push((input_nm, input_typ));
            }
        }
    }
    new_inputs
}

/// Replaces uses of records with all of their fields.
pub fn expand_record_io(input: &mut Vec<(String, Option<FullType>)>, output: &mut Vec<FullType>) {
    *input = replace_io(std::mem::take(input));
    *output = replace_io(
        std::mem::take(output)
            .into_iter()
            .map(|t| (String::new(), Some(t))),
    )
    .into_iter()
    .map(|(_, t)| t.unwrap())
    .collect();
}
