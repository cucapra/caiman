use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FuncletId, FunctionClassId, CommandId, StorageTypeId, TypeId,
};
use crate::assembly::context;
use crate::assembly::context::Context;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::{frontend, ir};
use serde_derive::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub funclet: ast::FuncletId,
    pub command: ast::CommandId,
}

pub fn unwrap_ffi_type(local: ast::TypeId) -> ast::FFIType {
    match local {
        TypeId::FFI(f) => f,
        TypeId::Local(_) => {
            unreachable!("Attempted to treat local type {:?} as an FFI type", &local)
        }
    }
}

pub fn todo_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => todo!(),
    }
}

pub fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => panic!("Invalid hole location"),
    }
}

pub fn reject_hole_clone<T>(node: &Hole<T>) -> T
where
    T: Clone,
{
    reject_hole(node.as_ref()).clone()
}

pub fn find_filled<T>(v: Vec<Hole<T>>) -> Vec<(usize, T)> {
    let mut result = Vec::new();
    for (index, hole) in v.into_iter().enumerate() {
        match hole {
            Some(value) => {
                result.push((index, value));
            }
            None => {}
        }
    }
    result
}

pub fn find_filled_hole<T>(h: Hole<Vec<Hole<T>>>) -> Vec<(usize, T)>
where
    T: Clone,
{
    match h {
        Some(v) => find_filled(v),
        None => Vec::new(),
    }
}

pub fn get_first<'a, T>(v: &'a Vec<T>, test: fn(&T) -> bool) -> Option<&'a T>
where
    T: Sized,
{
    for item in v {
        if test(item) {
            return Some(&item);
        }
    }
    None
}
