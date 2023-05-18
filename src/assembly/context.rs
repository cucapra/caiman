use crate::assembly::ast;
use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, NodeId, RemoteNodeId, StorageTypeId, TypeId,
    ValueFunctionId,
};
use crate::ir;
use debug_ignore::DebugIgnore;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

// #[derive(Debug)]
// pub struct Table<T>
// where
//     T: Eq + Hash + Debug + Clone,
// {
//     values: HashSet<T>,
//     indices: Vec<T>,
// }
//
// #[derive(Debug)]
// pub struct NodeTable {
//     // local names and return names such as [%out : i64] or whatever
//     pub local: Table<OperationId>,
//     pub returns: Table<OperationId>,
// }
//
// #[derive(Debug)]
// pub struct ValueFuncletConnections {
//     // stores connections of what refers to this value funclet
//     pub schedule_funclets: Vec<FuncletId>,
//     pub timeline_funclets: Vec<FuncletId>,
//     pub spatial_funclets: Vec<FuncletId>,
// }
//
// #[derive(Debug)]
// struct ValueExplicationInformation {
//     // explication locations are in the scheduling world
//     // maps from this node to the places it's been allocated on
//     scheduled_allocations: HashMap<FuncletId, OperationId>,
//
//     // indicates whether this operation has been written yet
//     // used primarily to add operations when needed
//     written: bool,
// }
//
// #[derive(Debug)]
// struct ValueFuncletData {
//     pub connections: ValueFuncletConnections,
//
//     // information about allocated value elements
//     explication_information: HashMap<OperationId, ValueExplicationInformation>,
//
//     // map from call index to output name for each call
//     call_outputs: HashMap<OperationId, HashMap<usize, OperationId>>,
// }
//
// #[derive(Debug)]
// struct ScheduleFuncletData {
//     // associated value funclet
//     value_funclet: FuncletId,
//     // map from the variables available to which node they are allocated
//     allocations: HashMap<OperationId, OperationId>,
//     // list of explication holes found, by index
//     explication_holes: Vec<usize>,
// }
//
// #[derive(Debug, Clone)]
// pub enum FuncletLocation {
//     Local,
//     ValueFunction,
//     External,
// }
//
// #[derive(Debug, Clone)]
// pub enum Location {
//     FFI(usize),
//     Local(usize),
// }
//
// #[derive(Debug, Clone)]
// pub enum NodeType {
//     // Keeps track of internal names vs return names
//     Local(usize),
//     Return(usize),
// }
//
// impl Location {
//     pub fn unpack(&self) -> usize {
//         match self {
//             Location::Local(u) => *u,
//             Location::FFI(u) => *u,
//             Location::FFI(u) => *u,
//         }
//     }
// }
//
// pub struct FuncletInformation {
//     location: FuncletLocation,
//     index: usize,
// }
//
// #[derive(Debug)]
// pub struct FuncletIndices {
//     external_funclet_table: Table<ExternalFunctionId>,
//     local_funclet_table: Table<FuncletId>,
//     value_function_table: Table<ValueFunctionId>,
//     funclet_kind_map: HashMap<String, FuncletLocation>,
// }
//
// #[derive(Debug)]
// struct MetaData {
//     name_index: usize,
// }
//
// #[derive(Debug)]
// pub struct Context {
//     pub ffi_type_table: Table<FFIType>,
//     pub local_type_table: Table<String>,
//     pub variable_map: HashMap<FuncletId, NodeTable>,
//     // where we currently are in the AST, using names
//     // optional cause we may not have started traversal
//     pub location: ast::RemoteNodeId,
//     pub funclet_indices: FuncletIndices,
//
//     // information found about a given value funclet
//     value_explication_data: HashMap<FuncletId, ValueFuncletData>,
//     // information found about a given schedule funclet
//     schedule_explication_data: HashMap<FuncletId, ScheduleFuncletData>,
//     meta_data: MetaData,
//     // connections from a given funclet
// }
//
// // a Table is basically a vector with no dupes
// impl<T> Table<T>
// where
//     T: Eq + Hash + Debug + Clone,
// {
//     pub fn new() -> Table<T> {
//         Table {
//             values: HashSet::new(),
//             indices: Vec::new(),
//         }
//     }
//
//     pub fn contains(&mut self, val: &T) -> bool {
//         self.values.contains(val)
//     }
//
//     pub fn dummy_push(&mut self, val: T) {
//         // Add unnamed element for indexing
//         self.indices.push(val);
//     }
//
//     pub fn push(&mut self, val: T) {
//         let msg = format!("Duplicate add of {:?}", val);
//         if !self.try_push(val) {
//             panic!(msg)
//         }
//     }
//
//     pub fn try_push(&mut self, val: T) -> bool {
//         self.indices.push(val.clone());
//         self.values.insert(val)
//     }
//
//     pub fn insert(&mut self, index: usize, val: T) {
//         if self.values.contains(&val) {
//             panic!("Duplicate add of {:?}", val)
//         }
//         self.values.insert(val.clone());
//         self.indices.insert(index, val);
//     }
//
//     pub fn get(&self, val: &T) -> Option<usize> {
//         // no need to actually check the Hashset, that's just to avoid dupes
//         for item in itertools::enumerate(&self.indices) {
//             if item.1 == val {
//                 return Some(item.0);
//             }
//         }
//         return None;
//     }
//
//     pub fn get_index(&self, val: &T) -> Option<usize> {
//         self.get(val)
//     }
//
//     pub fn get_at_index(&self, index: usize) -> Option<&T> {
//         if index >= self.indices.len() {
//             None
//         } else {
//             Some(&self.indices[index])
//         }
//     }
//
//     pub fn len(&mut self) -> usize {
//         return self.indices.len();
//     }
// }
//
// pub fn fresh_location() -> ast::RemoteNodeId {
//     ast::RemoteNodeId {
//         funclet_name: FuncletId("".to_string()),
//         node_name: OperationId("".to_string()),
//     }
// }
//
// impl NodeTable {
//     pub fn new() -> NodeTable {
//         NodeTable {
//             local: Table::new(),
//             returns: Table::new(),
//         }
//     }
// }
//
// impl ValueFuncletConnections {
//     pub fn new() -> ValueFuncletConnections {
//         ValueFuncletConnections {
//             schedule_funclets: Vec::new(),
//             timeline_funclets: Vec::new(),
//             spatial_funclets: Vec::new(),
//         }
//     }
// }
//
// impl ValueExplicationInformation {
//     pub fn new() -> ValueExplicationInformation {
//         ValueExplicationInformation {
//             scheduled_allocations: HashMap::new(),
//             written: false,
//         }
//     }
// }
//
// impl ValueFuncletData {
//     pub fn new() -> ValueFuncletData {
//         ValueFuncletData {
//             connections: ValueFuncletConnections::new(),
//             explication_information: HashMap::new(),
//             call_outputs: HashMap::new(),
//         }
//     }
//     pub fn allocate(
//         &mut self,
//         value_node: OperationId,
//         schedule_funclet: FuncletId,
//         schedule_node: OperationId,
//     ) {
//         self.explication_information
//             .entry(value_node)
//             .or_insert(ValueExplicationInformation::new())
//             .scheduled_allocations
//             .insert(schedule_funclet, schedule_node);
//     }
// }
//
// impl ScheduleFuncletData {
//     pub fn new(value_funclet: FuncletId) -> ScheduleFuncletData {
//         ScheduleFuncletData {
//             value_funclet,
//             allocations: HashMap::new(),
//             explication_holes: Vec::new(),
//         }
//     }
//     pub fn allocate(&mut self, schedule_node: OperationId, value_node: OperationId) {
//         self.allocations.insert(schedule_node, value_node);
//     }
// }
//
// impl FuncletIndices {
//     pub fn new() -> FuncletIndices {
//         FuncletIndices {
//             external_funclet_table: Table::new(),
//             local_funclet_table: Table::new(),
//             value_function_table: Table::new(),
//             funclet_kind_map: HashMap::new(),
//         }
//     }
//
//     pub fn insert(&mut self, name: String, location: FuncletLocation) {
//         match location {
//             FuncletLocation::Local => self.local_funclet_table.push(FuncletId(name.clone())),
//             FuncletLocation::External => self
//                 .external_funclet_table
//                 .push(ExternalFunctionId(name.clone())),
//             FuncletLocation::ValueFunction => self
//                 .value_function_table
//                 .push(ValueFunctionId(name.clone())),
//         }
//         self.funclet_kind_map.insert(name, location);
//     }
//
//     pub fn get_index(&self, name: &FuncletId) -> Option<usize> {
//         self.local_funclet_table.get(name)
//     }
//
//     pub fn get_loc(&self, name: &String) -> Option<&FuncletLocation> {
//         self.funclet_kind_map.get(name)
//     }
//
//     pub fn get_funclet(&self, name: &String) -> Option<usize> {
//         self.funclet_kind_map.get(name).and_then(|x| match x {
//             FuncletLocation::Local => self.local_funclet_table.get(&FuncletId(name.clone())),
//             FuncletLocation::External => self
//                 .external_funclet_table
//                 .get(&ExternalFunctionId(name.clone())),
//             FuncletLocation::ValueFunction => self
//                 .value_function_table
//                 .get(&ValueFunctionId(name.clone())),
//         })
//     }
// }
//
// impl MetaData {
//     pub fn new(name_index: usize) -> MetaData {
//         MetaData { name_index }
//     }
//     pub fn next_name(&mut self) -> String {
//         self.name_index += 1;
//         format!("~{}", self.name_index)
//     }
// }
//
// impl Context {
//     pub fn new(program: &ast::Program, name_index: usize) -> Context {
//         let mut context = Context {
//             value_explication_data: HashMap::new(),
//             schedule_explication_data: HashMap::new(),
//             meta_data: MetaData::new(name_index),
//             ffi_type_table: Table::new(),
//             local_type_table: Table::new(),
//             funclet_indices: FuncletIndices::new(),
//             variable_map: HashMap::new(),
//             location: fresh_location(),
//         };
//         context.setup_context(program);
//         context
//     }
//
//     // Take a pass over the program to construct the initial context
//     // We only do this once and assume the invariants are maintained by construction
//     // Note that a context without this makes little sense, so we can't build an "empty context"
//     fn setup_context(&mut self, program: &ast::Program) {
//         for typ in &program.types {
//             match typ {
//                 ast::TypeDecl::FFI(t) => self.ffi_type_table.push(t.clone()),
//                 ast::TypeDecl::Local(t) => self.local_type_table.push(t.name.clone()),
//             }
//         }
//         for funclet in &program.funclets {
//             match funclet {
//                 ast::FuncletDef::External(f) => {
//                     self.funclet_indices
//                         .insert(f.name.clone(), FuncletLocation::External);
//                 }
//                 ast::FuncletDef::Local(f) => {
//                     match f.kind {
//                         ir::FuncletKind::Value => {
//                             self.value_explication_data
//                                 .insert(f.header.name.clone(), ValueFuncletData::new());
//                         }
//                         _ => {}
//                     };
//                     self.funclet_indices
//                         .insert(f.header.name.0.clone(), FuncletLocation::Local);
//                     let mut node_table = NodeTable::new();
//                     for command in &f.commands {
//                         match command {
//                             None => {}
//                             Some(ast::NamedNode { node, name }) => {
//                                 // a bit sketchy, but if we only correct this here, we should be ok
//                                 // basically we never rebuild the context
//                                 // and these names only matter for this context anyway
//                                 node_table.local.push(name.clone());
//                             }
//                         }
//                     }
//                     for (n, _) in &f.header.ret {
//                         match n {
//                             None => {}
//                             Some(name) => {
//                                 node_table.returns.push(name.clone());
//                             }
//                         }
//                     }
//                     self.variable_map.insert(f.header.name.clone(), node_table);
//                 }
//                 ast::FuncletDef::ValueFunction(f) => {
//                     self.funclet_indices
//                         .insert(f.name.clone(), FuncletLocation::ValueFunction);
//                 }
//             }
//         }
//     }
//
//     pub fn setup_schedule_data(&mut self, value_funclet: FuncletId) {
//         self.value_explication_data
//             .get_mut(&value_funclet)
//             .unwrap()
//             .connections
//             .schedule_funclets
//             .push(self.location.funclet_name.clone());
//         self.schedule_explication_data.insert(
//             self.location.funclet_name.clone(),
//             ScheduleFuncletData::new(value_funclet),
//         );
//     }
//
//     pub fn reset_location(&mut self) {
//         self.location = fresh_location()
//     }
//
//     pub fn ffi_type_id(&self, name: &ast::FFIType) -> usize {
//         match self.ffi_type_table.get_index(name) {
//             Some(i) => i,
//             None => panic!("Un-indexed FFI type {:?}", name),
//         }
//     }
//
//     pub fn local_type_id(&self, name: &String) -> usize {
//         match self.local_type_table.get_index(name) {
//             Some(t) => t,
//             None => panic!("Unknown local type {:?}", name),
//         }
//     }
//
//     pub fn loc_type_id(&self, typ: &ast::Type) -> usize {
//         match typ {
//             ast::Type::FFI(ft) => self.ffi_type_id(&ft),
//             ast::Type::Local(s) => self.local_type_id(&s),
//         }
//     }
//
//     pub fn remote_node_id(&self, funclet: &FuncletId, var: &OperationId) -> usize {
//         match self.variable_map.get(funclet) {
//             Some(f) => match f.local.get(var) {
//                 Some(v) => v,
//                 None => panic!("Unknown local name {} in funclet {}", var, funclet),
//             },
//             None => panic!("Unknown funclet name {}", funclet),
//         }
//     }
//
//     pub fn remote_return_id(&self, funclet: &FuncletId, var: &OperationId) -> usize {
//         match self.variable_map.get(funclet) {
//             Some(f) => match f.returns.get_index(var) {
//                 Some(v) => v,
//                 None => panic!("Unknown return name {} in funclet {}", var, funclet),
//             },
//             None => panic!("Unknown funclet name {}", funclet),
//         }
//     }
//
//     pub fn node_from_id(&self, index: usize) -> OperationId {
//         self.variable_map
//             .get(&self.location.funclet_name)
//             .unwrap()
//             .local
//             .get_at_index(index)
//             .unwrap()
//             .clone()
//     }
//
//     pub fn node_id(&self, var: &OperationId) -> usize {
//         let funclet = &self.location.funclet_name;
//         match self.variable_map.get(funclet).unwrap().local.get_index(var) {
//             Some(v) => v,
//             None => panic!("Unknown variable name {:?} in funclet {:?}", var, &funclet),
//         }
//     }
//
//     pub fn return_id(&self, var: &OperationId) -> usize {
//         let funclet = &self.location.funclet_name;
//         match self
//             .variable_map
//             .get(funclet)
//             .unwrap()
//             .returns
//             .get_index(var)
//         {
//             Some(v) => v,
//             None => panic!("Unknown return name {:?} in funclet {:?}", var, &funclet),
//         }
//     }
//
//     pub fn remote_id(&self, funclet: &FuncletId, var: &OperationId) -> ir::RemoteNodeId {
//         ir::RemoteNodeId {
//             funclet_id: self
//                 .funclet_indices
//                 .local_funclet_table
//                 .get(funclet)
//                 .unwrap()
//                 .clone(),
//             node_id: self.remote_node_id(funclet, var),
//         }
//     }
//
//     fn allocation_name(&mut self) -> String {
//         self.meta_data.name_index += 1;
//         format!("${}", self.meta_data.name_index)
//     }
//
//     pub fn add_allocation(&mut self, value_node: OperationId, schedule_node: OperationId) {
//         let schedule_funclet = self.location.funclet_name.clone();
//         let value_funclet = &self
//             .schedule_explication_data
//             .get(&schedule_funclet)
//             .unwrap()
//             .value_funclet;
//
//         self.variable_map
//             .get_mut(&schedule_funclet)
//             .unwrap()
//             .local
//             .try_push(schedule_node.clone());
//
//         // unwrap explicitly cause we assume funclet data are setup earlier
//         self.value_explication_data
//             .get_mut(value_funclet)
//             .unwrap()
//             .allocate(
//                 value_node.clone(),
//                 schedule_funclet.clone(),
//                 schedule_node.clone(),
//             );
//
//         self.schedule_explication_data
//             .get_mut(&schedule_funclet)
//             .unwrap()
//             .allocate(schedule_node, value_node);
//     }
//
//     // get allocations of the associated value node
//     pub fn get_schedule_allocations(
//         &self,
//         funclet: &FuncletId,
//         node: &OperationId,
//     ) -> Option<&HashMap<FuncletId, OperationId>> {
//         self.value_explication_data.get(funclet).and_then(|f| {
//             f.explication_information
//                 .get(node)
//                 .map(|n| &n.scheduled_allocations)
//         })
//     }
//
//     pub fn get_current_schedule_allocation(&self, node: &OperationId) -> Option<&OperationId> {
//         self.get_current_value_funclet().as_ref().and_then(|vf| {
//             self.get_schedule_allocations(&vf, node)
//                 .unwrap()
//                 .get(&self.location.funclet_name)
//         })
//     }
//
//     // get what the associated schedule node is allocating
//     pub fn get_value_allocation(
//         &self,
//         funclet: &FuncletId,
//         node: &OperationId,
//     ) -> Option<&ast::OperationId> {
//         self.schedule_explication_data
//             .get(funclet)
//             .and_then(|f| f.allocations.get(node))
//     }
//
//     pub fn get_current_value_funclet(&self) -> Option<&FuncletId> {
//         self.schedule_explication_data
//             .get(&self.location.funclet_name)
//             .map(|f| &f.value_funclet)
//     }
//
//     pub fn next_name(&mut self) -> String {
//         self.meta_data.next_name()
//     }
// }
