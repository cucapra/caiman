use crate::assembly::ast;
use crate::assembly::ast::FFIType;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FuncletId, FunctionClassId, NodeId, StorageTypeId, TypeId,
};
use crate::assembly::context;
use crate::assembly::context::Context;
use crate::assembly::parser;
use crate::ir::ffi;
use crate::{frontend, ir};
use std::any::Any;
use std::collections::HashMap;

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
        Some(v) => find_filled(v.into_vec()),
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
