use crate::ir;
use naga::{
    AddressSpace, Arena, AtomicFunction, Binding, Block, Constant, ConstantInner, EntryPoint,
    Expression, Function, FunctionArgument, FunctionResult, GlobalVariable, Handle, ImageQuery,
    LocalVariable, Module, Range, RayQueryFunction, ResourceBinding, SampleLevel, ShaderStage,
    Span, Statement, SwitchCase, Type, UniqueArena,
};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::error::Error;

/// Specifies the "form" of a GPU kernel dispatch.
#[derive(Debug)]
pub struct FuseSource<'a> {
    /// The shader module implementing this kernel dispatch.
    shader_module: &'a ShaderModule,
    /// The entry point used for this kernel dispatch.
    entry_point: &'a str,
}

/// Specifies where a resource should be remapped to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FusedResource {
    /// The resource should live in `@group([group]) @binding([binding])`
    Binding { group: u32, binding: u32 },
    /// The resource should live in a global variable with the given index.
    /// (The index is **only** used to differentiate different global destinations.)
    Global(usize),
}
impl FusedResource {
    fn to_naga_binding(&self) -> Option<ResourceBinding> {
        match self {
            Self::Binding { group, binding } => Some(ResourceBinding {
                group: *group,
                binding: *binding,
            }),
            Self::Global(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct FuseDescriptor<'a> {
    /// The modules to fuse into a single kernel dispatch, ordered by execution.
    sources: &'a [FuseSource<'a>],
    /// Each entry `(i, g, b) -> (g', b')` specifies that `group=g`, `binding=b` of `modules[i]`
    /// should be placed into the specified resource.
    /// The map is deliberately non-injective to allow fusing bound resources.
    resources: &'a HashMap<(usize, u32, u32), FusedResource>,
}

#[derive(Debug)]
pub struct ShaderModule {
    module: naga::Module,
}

impl ShaderModule {
    pub fn from_wgsl(text: &str) -> Result<Self, Box<dyn Error>> {
        let module = naga::front::wgsl::parse_str(text)?;
        Ok(Self { module })
    }
    pub fn from_glsl(text: &str) -> Result<Self, Box<dyn Error>> {
        let mut parser = naga::front::glsl::Frontend::default();
        let options = naga::front::glsl::Options::from(naga::ShaderStage::Compute);
        match parser.parse(&options, text) {
            Ok(module) => Ok(Self { module }),
            Err(errors) => {
                // Just report the first error for now.
                // Could do something better here in the future, but there's a lot of
                // error handling work that needs to be done beforehand...
                Err(Box::new(errors.into_iter().next().unwrap()))
            }
        }
    }
    pub fn emit_wgsl(&self) -> String {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        );
        let module_info = match validator.validate(&self.module) {
            Err(why) => panic!("Error while validating WGSL: {}", why),
            Ok(module_info) => module_info,
        };
        match naga::back::wgsl::write_string(
            &self.module,
            &module_info,
            naga::back::wgsl::WriterFlags::EXPLICIT_TYPES,
        ) {
            Err(why) => panic!("Error while emitting WGSL: {}", why),
            Ok(text) => text,
        }
    }

    /// Fuses all compute kernels in `modules` into a single kernel. The execution order
    /// is defined by the order the kernels appear within `modules`.
    pub fn fuse(desc: FuseDescriptor) -> ShaderModule {
        let mut types = naga::UniqueArena::new();
        let mut constants = naga::Arena::new();
        let mut resource_locations = HashMap::new(); // maps (group, binding) to GlobalVariable
        let mut global_variables = naga::Arena::new();
        let mut functions = naga::Arena::new();
        let mut entry_point_arg_map = HashMap::new(); // maps entry point arguments to expressions
        let mut entry_point = None;

        for (index, source) in desc.sources.iter().enumerate() {
            let mut fuser = Fuser {
                module: &source.shader_module.module,
                entry_point: &source.entry_point,
                resources: desc.resources,
                index,
            };
            let type_map = fuser.fuse_types(&mut types);
            let cst_map = fuser.fuse_constants(&type_map, &mut constants);
            let gv_map = fuser.fuse_global_variables(
                &type_map,
                &cst_map,
                &mut resource_locations,
                &mut global_variables,
            );
            let fn_map = fuser.fuse_functions(&type_map, &cst_map, &gv_map, &mut functions);
            fuser.fuse_entry_points(
                &type_map,
                &cst_map,
                &gv_map,
                &fn_map,
                &mut functions,
                &mut entry_point_arg_map,
                &mut entry_point,
            );
        }
        return ShaderModule {
            module: Module {
                types,
                special_types: naga::SpecialTypes {
                    ray_desc: None,
                    ray_intersection: None,
                },
                constants,
                global_variables,
                functions,
                entry_points: vec![entry_point.expect("tried to merge zero kernels")],
            },
        };
    }
}

type Remap<T> = HashMap<Handle<T>, Handle<T>>;

// doesn't need to exist, but cuts down on the verbosity of arguments.
struct Fuser<'a> {
    module: &'a Module,
    entry_point: &'a str,
    resources: &'a HashMap<(usize, u32, u32), FusedResource>,
    index: usize,
}

impl<'a> Fuser<'a> {
    fn rename(&self, name: &Option<String>) -> Option<String> {
        return name
            .as_ref()
            .map(|s| format!("fused_{:02}__{}", self.index, s));
    }
    fn fuse_types(&self, types: &mut UniqueArena<Type>) -> Remap<Type> {
        let mut type_map = Remap::new();
        for (old_handle, old_ty) in self.module.types.iter() {
            let span = self.module.types.get_span(old_handle);
            let new_ty = Type {
                name: self.rename(&old_ty.name),
                inner: old_ty.inner.clone(),
            };
            let new_handle = types.insert(new_ty, span);
            type_map.insert(old_handle, new_handle);
        }
        // we shouldn't be raytracing in a compute shader
        assert!(self.module.special_types.ray_desc.is_none());
        assert!(self.module.special_types.ray_intersection.is_none());
        return type_map;
    }
    fn fuse_constants(
        &self,
        type_map: &Remap<Type>,
        constants: &mut Arena<Constant>,
    ) -> Remap<Constant> {
        let mut cst_map = Remap::new();
        for (old_handle, old_cst) in self.module.constants.iter() {
            let span = self.module.constants.get_span(old_handle);
            // don't know how to handle specialization constants
            assert!(old_cst.specialization.is_none());

            let new_cst_inner = match &old_cst.inner {
                ConstantInner::Scalar { .. } => old_cst.inner.clone(),
                ConstantInner::Composite { ty, components } => {
                    ConstantInner::Composite {
                        ty: type_map[&ty],
                        components: components.clone(), // we'll fix this later
                    }
                }
            };
            let new_cst = naga::Constant {
                name: self.rename(&old_cst.name),
                specialization: None,
                inner: new_cst_inner,
            };
            let new_handle = constants.append(new_cst, span);
            cst_map.insert(old_handle, new_handle);
        }
        // go back & fix up the composite constants we created
        for &new_handle in cst_map.values() {
            let cst = constants.get_mut(new_handle);
            let cst_inner = &mut cst.inner;
            if let ConstantInner::Composite { components, .. } = cst_inner {
                for comp in components.iter_mut() {
                    *comp = cst_map[comp];
                }
            }
        }
        return cst_map;
    }

    fn fuse_global_variables(
        &mut self,
        type_map: &Remap<Type>,
        cst_map: &Remap<Constant>,
        resource_locations: &mut HashMap<FusedResource, Handle<GlobalVariable>>,
        global_variables: &mut Arena<GlobalVariable>,
    ) -> Remap<GlobalVariable> {
        let mut gv_map = Remap::new();
        for (old_handle, old_gv) in self.module.global_variables.iter() {
            let mut dest = None;

            if let Some(ResourceBinding { group, binding }) = &old_gv.binding {
                let location = self
                    .resources
                    .get(&(self.index, *group, *binding))
                    .expect("no resource relocation info for binding");
                dest = Some(location);

                if let Some(&new_handle) = resource_locations.get(&location) {
                    use AddressSpace::{Private, Storage};
                    // There's an existing binding. For the time being we won't validate
                    // whether the types are compatible, we'll just fixup buffer flags
                    let new_gv = global_variables.get_mut(new_handle);
                    match (&mut new_gv.space, &old_gv.space) {
                        #[rustfmt::skip]
                        (Storage { access: dst }, Storage { access: src }) => *dst |= *src,
                        (Private, _) => (), // moving a binding into a global
                        (a, b) => assert_eq!(a, b),
                    }
                    gv_map.insert(old_handle, new_handle);
                    continue;
                }
            }

            // If we're turning bindings into globals, we need to fix up their address space.
            let new_space = match dest {
                Some(FusedResource::Global(_)) => AddressSpace::Private,
                _ => old_gv.space.clone(),
            };

            let span = self.module.global_variables.get_span(old_handle);
            let new_gv = GlobalVariable {
                name: self.rename(&old_gv.name),
                space: new_space,
                binding: dest.map(FusedResource::to_naga_binding).flatten(),
                ty: type_map[&old_gv.ty],
                init: old_gv.init.map(|old_cst| cst_map[&old_cst]),
            };
            let new_handle = global_variables.append(new_gv, span);
            gv_map.insert(old_handle, new_handle);
            // If we just added a new resource binding, we need to make sure later iterations
            // know where to find it.
            if let Some(&location) = dest {
                resource_locations.insert(location, new_handle);
            }
        }
        return gv_map;
    }

    fn update_sample_level(old_level: &SampleLevel, expr_map: &Remap<Expression>) -> SampleLevel {
        match old_level {
            SampleLevel::Auto => SampleLevel::Auto,
            SampleLevel::Zero => SampleLevel::Zero,
            SampleLevel::Exact(old_expr) => SampleLevel::Exact(expr_map[old_expr]),
            SampleLevel::Bias(old_expr) => SampleLevel::Bias(expr_map[old_expr]),
            SampleLevel::Gradient { x, y } => SampleLevel::Gradient {
                x: expr_map[x],
                y: expr_map[y],
            },
        }
    }
    fn update_image_query(old_query: &ImageQuery, expr_map: &Remap<Expression>) -> ImageQuery {
        match old_query {
            ImageQuery::Size { level } => ImageQuery::Size {
                level: level.as_ref().map(|old_expr| expr_map[old_expr]),
            },
            ImageQuery::NumLevels => ImageQuery::NumLevels,
            ImageQuery::NumLayers => ImageQuery::NumLayers,
            ImageQuery::NumSamples => ImageQuery::NumSamples,
        }
    }
    fn update_expr(
        old_expr: &Expression,
        type_map: &Remap<Type>,
        cst_map: &Remap<Constant>,
        gv_map: &Remap<GlobalVariable>,
        fn_map: &Remap<Function>,
        lv_map: &Remap<LocalVariable>,
        expr_map: &Remap<Expression>,
    ) -> Expression {
        match old_expr {
            Expression::Access { base, index } => Expression::Access {
                base: expr_map[base],
                index: expr_map[index],
            },
            Expression::AccessIndex { base, index } => Expression::AccessIndex {
                base: expr_map[base],
                index: *index,
            },
            Expression::Constant(old_cst) => Expression::Constant(cst_map[old_cst]),
            Expression::Splat { size, value } => Expression::Splat {
                size: *size,
                value: expr_map[value],
            },
            Expression::Swizzle {
                size,
                vector,
                pattern,
            } => Expression::Swizzle {
                size: *size,
                vector: expr_map[vector],
                pattern: *pattern,
            },
            Expression::Compose { ty, components } => Expression::Compose {
                ty: type_map[ty],
                components: components
                    .iter()
                    .map(|old_expr| expr_map[old_expr])
                    .collect(),
            },
            Expression::FunctionArgument(index) => Expression::FunctionArgument(*index),
            Expression::GlobalVariable(global) => Expression::GlobalVariable(gv_map[global]),
            Expression::LocalVariable(local) => Expression::LocalVariable(lv_map[local]),
            Expression::Load { pointer } => Expression::Load {
                pointer: expr_map[pointer],
            },
            Expression::ImageSample {
                image,
                sampler,
                gather,
                coordinate,
                array_index,
                offset,
                level,
                depth_ref,
            } => Expression::ImageSample {
                image: expr_map[image],
                sampler: expr_map[sampler], // kind of weird that samplers are expressions
                gather: gather.clone(),
                coordinate: expr_map[coordinate],
                array_index: array_index.as_ref().map(|old_expr| expr_map[old_expr]),
                offset: offset.as_ref().map(|old_cst| cst_map[old_cst]),
                level: Self::update_sample_level(level, expr_map),
                depth_ref: depth_ref.as_ref().map(|old_expr| expr_map[old_expr]),
            },
            Expression::ImageLoad {
                image,
                coordinate,
                array_index,
                sample,
                level,
            } => Expression::ImageLoad {
                image: expr_map[image],
                coordinate: expr_map[coordinate],
                array_index: array_index.as_ref().map(|old_expr| expr_map[old_expr]),
                sample: sample.as_ref().map(|old_expr| expr_map[old_expr]),
                level: level.as_ref().map(|old_expr| expr_map[old_expr]),
            },
            Expression::ImageQuery { image, query } => Expression::ImageQuery {
                image: expr_map[image],
                query: Self::update_image_query(query, expr_map),
            },
            Expression::Unary { op, expr } => Expression::Unary {
                op: op.clone(),
                expr: expr_map[expr],
            },
            Expression::Binary { op, left, right } => Expression::Binary {
                op: op.clone(),
                left: expr_map[left],
                right: expr_map[right],
            },
            Expression::Select {
                condition,
                accept,
                reject,
            } => Expression::Select {
                condition: expr_map[condition],
                accept: expr_map[accept],
                reject: expr_map[reject],
            },
            Expression::Derivative { axis, ctrl, expr } => Expression::Derivative {
                axis: axis.clone(),
                ctrl: ctrl.clone(),
                expr: expr_map[expr],
            },
            Expression::Relational { fun, argument } => Expression::Relational {
                fun: fun.clone(),
                argument: expr_map[argument],
            },
            Expression::Math {
                fun,
                arg,
                arg1,
                arg2,
                arg3,
            } => Expression::Math {
                fun: fun.clone(),
                arg: expr_map[arg],
                arg1: arg1.as_ref().map(|old_expr| expr_map[old_expr]),
                arg2: arg2.as_ref().map(|old_expr| expr_map[old_expr]),
                arg3: arg3.as_ref().map(|old_expr| expr_map[old_expr]),
            },
            Expression::As {
                expr,
                kind,
                convert,
            } => Expression::As {
                expr: expr_map[expr],
                kind: kind.clone(),
                convert: convert.clone(),
            },
            Expression::CallResult(fun) => Expression::CallResult(fn_map[fun]),
            Expression::AtomicResult { ty, comparison } => Expression::AtomicResult {
                ty: type_map[ty],
                comparison: *comparison,
            },
            Expression::ArrayLength(expr) => Expression::ArrayLength(expr_map[expr]),
            Expression::RayQueryProceedResult => Expression::RayQueryProceedResult,
            Expression::RayQueryGetIntersection { query, committed } => {
                Expression::RayQueryGetIntersection {
                    query: expr_map[query],
                    committed: *committed,
                }
            }
        }
    }

    fn update_block(
        old_block: &Block,
        fn_map: &Remap<Function>,
        expr_map: &Remap<Expression>,
    ) -> Block {
        let mut new_block = Block::with_capacity(old_block.len());
        for (old_stmt, &span) in old_block.span_iter() {
            let new_stmt = Self::update_stmt(old_stmt, fn_map, expr_map);
            new_block.push(new_stmt, span);
        }
        return new_block;
    }

    fn update_atomic_fn(old_fun: &AtomicFunction, expr_map: &Remap<Expression>) -> AtomicFunction {
        match old_fun {
            AtomicFunction::Add => AtomicFunction::Add,
            AtomicFunction::Subtract => AtomicFunction::Subtract,
            AtomicFunction::And => AtomicFunction::And,
            AtomicFunction::ExclusiveOr => AtomicFunction::ExclusiveOr,
            AtomicFunction::InclusiveOr => AtomicFunction::InclusiveOr,
            AtomicFunction::Min => AtomicFunction::Min,
            AtomicFunction::Max => AtomicFunction::Max,
            AtomicFunction::Exchange { compare } => AtomicFunction::Exchange {
                compare: compare.as_ref().map(|x| expr_map[x]),
            },
        }
    }

    fn update_ray_query_fn(
        old_rqf: &RayQueryFunction,
        expr_map: &Remap<Expression>,
    ) -> RayQueryFunction {
        match old_rqf {
            RayQueryFunction::Initialize {
                acceleration_structure,
                descriptor,
            } => RayQueryFunction::Initialize {
                acceleration_structure: expr_map[acceleration_structure],
                descriptor: expr_map[descriptor],
            },
            RayQueryFunction::Proceed { result } => RayQueryFunction::Proceed {
                result: expr_map[result],
            },
            RayQueryFunction::Terminate => RayQueryFunction::Terminate,
        }
    }

    fn update_stmt(
        old_stmt: &Statement,
        fn_map: &Remap<Function>,
        expr_map: &Remap<Expression>,
    ) -> Statement {
        match old_stmt {
            Statement::Emit(range) => {
                // HACK:  Naga doesn't want people seeing range endpoints for some reason. (???)
                // So... bye bye type system
                let worst_thing_ever = ron::to_string(range).expect("seriously");
                let really_awful: std::ops::Range<u32> =
                    ron::from_str(&worst_thing_ever).expect("this should be a crime");
                let start_str = really_awful.start.to_string();
                let start: Handle<Expression> = ron::from_str(&start_str).expect("sorry");
                let end_str = really_awful.end.to_string();
                let end: Handle<Expression> = ron::from_str(&end_str).expect("sorry sorry");
                // The funniest thing is that THIS is the part that might break.
                let new_range = Range::new_from_bounds(expr_map[&start], expr_map[&end]);
                Statement::Emit(new_range)
            }
            Statement::Block(block) => {
                Statement::Block(Self::update_block(block, fn_map, expr_map))
            }
            Statement::If {
                condition,
                accept,
                reject,
            } => Statement::If {
                condition: expr_map[condition],
                accept: Self::update_block(accept, fn_map, expr_map),
                reject: Self::update_block(reject, fn_map, expr_map),
            },
            Statement::Switch { selector, cases } => Statement::Switch {
                selector: expr_map[selector],
                cases: cases
                    .iter()
                    .map(|old_case| SwitchCase {
                        value: old_case.value.clone(),
                        body: Self::update_block(&old_case.body, fn_map, expr_map),
                        fall_through: old_case.fall_through,
                    })
                    .collect(),
            },
            Statement::Loop {
                body,
                continuing,
                break_if,
            } => Statement::Loop {
                body: Self::update_block(body, fn_map, expr_map),
                continuing: Self::update_block(body, fn_map, expr_map),
                break_if: break_if.as_ref().map(|expr| expr_map[expr]),
            },
            Statement::Break => Statement::Break,
            Statement::Continue => Statement::Continue,
            Statement::Return { value } => Statement::Return {
                value: value.as_ref().map(|expr| expr_map[expr]),
            },

            Statement::Kill => Statement::Kill,
            Statement::Barrier(barrier) => Statement::Barrier(barrier.clone()),
            Statement::Store { pointer, value } => Statement::Store {
                pointer: expr_map[pointer],
                value: expr_map[value],
            },
            Statement::ImageStore {
                image,
                coordinate,
                array_index,
                value,
            } => Statement::ImageStore {
                image: expr_map[image],
                coordinate: expr_map[coordinate],
                array_index: array_index.as_ref().map(|expr| expr_map[expr]),
                value: expr_map[value],
            },
            Statement::Atomic {
                pointer,
                fun,
                value,
                result,
            } => Statement::Atomic {
                pointer: expr_map[pointer],
                fun: Self::update_atomic_fn(fun, expr_map),
                value: expr_map[value],
                result: expr_map[result],
            },
            Statement::Call {
                function,
                arguments,
                result,
            } => Statement::Call {
                function: fn_map[function],
                arguments: arguments.iter().map(|arg| expr_map[arg]).collect(),
                result: result.as_ref().map(|res| expr_map[res]),
            },
            Statement::RayQuery { query, fun } => Statement::RayQuery {
                query: expr_map[query],
                fun: Self::update_ray_query_fn(fun, expr_map),
            },
        }
    }

    fn remap_fn(
        &self,
        old_fn: &Function,
        arg_action: impl FnMut(&FunctionArgument) -> FunctionArgument,
        type_map: &Remap<Type>,
        cst_map: &Remap<Constant>,
        gv_map: &Remap<GlobalVariable>,
        fn_map: &Remap<Function>,
    ) -> Function {
        let new_name = self.rename(&old_fn.name);
        let new_arguments: Vec<_> = old_fn.arguments.iter().map(arg_action).collect();
        let new_result = old_fn.result.as_ref().map(|res| FunctionResult {
            ty: type_map[&res.ty],
            binding: res.binding.clone(),
        });
        let mut lv_map = Remap::new();
        let mut new_local_variables = Arena::new();
        for (old_handle, old_local) in old_fn.local_variables.iter() {
            let span = old_fn.local_variables.get_span(old_handle);
            let new_local = LocalVariable {
                // We shouldn't need to rename locals, but someone could name them something
                // dumb like __fused__0__myVector, so may as well be consistent
                name: self.rename(&old_local.name),
                ty: type_map[&old_local.ty],
                init: old_local.init.as_ref().map(|old_cst| cst_map[&old_cst]),
            };
            let new_handle = new_local_variables.append(new_local, span);
            lv_map.insert(old_handle, new_handle);
        }
        let mut expr_map = Remap::new();
        let mut new_expressions = Arena::new();
        for (old_handle, old_expr) in old_fn.expressions.iter() {
            let span = old_fn.expressions.get_span(old_handle);
            let new_expr = Self::update_expr(
                old_expr, type_map, cst_map, gv_map, &fn_map, &lv_map, &expr_map,
            );
            let new_handle = new_expressions.append(new_expr, span);
            expr_map.insert(old_handle, new_handle);
        }
        // TODO: We lose information here, but in order to do this properly we need to
        // add an indexmap and rustc_hash dependency since Naga uses those in its public
        // interface but doesn't re-export them
        let mut new_named_expressions = Default::default();
        let mut new_body = Self::update_block(&old_fn.body, &fn_map, &expr_map);

        return Function {
            name: new_name,
            arguments: new_arguments,
            result: new_result,
            local_variables: new_local_variables,
            expressions: new_expressions,
            named_expressions: new_named_expressions,
            body: new_body,
        };
    }

    fn fuse_functions(
        &self,
        type_map: &Remap<Type>,
        cst_map: &Remap<Constant>,
        gv_map: &Remap<GlobalVariable>,
        functions: &mut Arena<Function>,
    ) -> Remap<Function> {
        let mut fn_map = Remap::new();
        for (old_handle, old_fn) in self.module.functions.iter() {
            let span = self.module.functions.get_span(old_handle);
            let arg_action = |arg: &FunctionArgument| FunctionArgument {
                name: self.rename(&arg.name),
                ty: type_map[&arg.ty],
                binding: arg.binding.clone(),
            };
            let new_fn = self.remap_fn(old_fn, arg_action, type_map, cst_map, gv_map, &fn_map);
            let new_handle = functions.append(new_fn, span);
            fn_map.insert(old_handle, new_handle);
        }
        return fn_map;
    }

    /// Generates an empty entry point named "main" from `template`, inheriting its stage,
    /// workgroup size, and depth test status.
    fn empty_entry_point(template: &EntryPoint) -> EntryPoint {
        return EntryPoint {
            name: "main".to_owned(),
            stage: template.stage,
            workgroup_size: template.workgroup_size,
            early_depth_test: template.early_depth_test,
            function: Function {
                name: Some("main".to_owned()),
                arguments: Vec::new(),
                result: None,
                local_variables: Arena::new(),
                expressions: Arena::new(),
                named_expressions: Default::default(),
                body: Block::new(),
            },
        };
    }

    fn fuse_entry_points(
        &self,
        type_map: &Remap<Type>,
        cst_map: &Remap<Constant>,
        gv_map: &Remap<GlobalVariable>,
        fn_map: &Remap<Function>,
        functions: &mut Arena<Function>,
        entry_point_arg_map: &mut HashMap<Binding, Handle<Expression>>,
        fused_entry_point: &mut Option<EntryPoint>,
    ) {
        // We don't need to update fn_map since source entry points can't call each other.
        let old_ep = self
            .module
            .entry_points
            .iter()
            .find(|ep| ep.name == self.entry_point)
            .expect("kernel fusion: unknown entry point");

        let fused = fused_entry_point.get_or_insert_with(|| Self::empty_entry_point(old_ep));

        assert_eq!(
            &old_ep.stage, &fused.stage,
            "kernel fusion: kernels have incompatible stages"
        );
        assert_eq!(
            &old_ep.workgroup_size, &fused.workgroup_size,
            "kernel fusion: kernels specify different workgroup sizes"
        );
        assert_eq!(
            &old_ep.early_depth_test, &fused.early_depth_test,
            "kernel fusion: kernels differ in use of early depth test"
        );

        // the function arguments appearing in the fused kernel's entry point
        let mut arguments = Vec::new();

        let arg_action = |arg: &FunctionArgument| {
            let FunctionArgument { ty, binding, name } = arg;
            let binding = binding
                .to_owned()
                .expect("entry point argument which isn't not a binding?");
            let ty = type_map[ty];
            match entry_point_arg_map.entry(binding.clone()) {
                Entry::Vacant(ve) => {
                    let fused_arg = FunctionArgument {
                        name: name.to_owned(),
                        ty,
                        binding: Some(binding),
                    };
                    // Ensure that this is passed as an argument to the new function...
                    let fused_idx = u32::try_from(fused.function.arguments.len()).unwrap();
                    fused.function.arguments.push(fused_arg);
                    // Make an expression grabbing that argument..
                    let fused_arg_expr = fused
                        .function
                        .expressions
                        .append(Expression::FunctionArgument(fused_idx), Span::default());
                    // Add to the map so we can reuse the argument if it appears in later modules
                    ve.insert(fused_arg_expr);
                    arguments.push(fused_arg_expr);
                }
                Entry::Occupied(oe) => {
                    arguments.push(*oe.get());
                }
            }
            return FunctionArgument {
                name: name.to_owned(),
                ty,
                binding: None,
            };
        };

        let new_fn = self.remap_fn(
            &old_ep.function,
            arg_action,
            type_map,
            cst_map,
            gv_map,
            &fn_map,
        );
        // not sure how we're supposed to find the span for an entry point
        let new_handle = functions.append(new_fn, Span::default());
        let fused_call_stmt = Statement::Call {
            function: new_handle,
            arguments,
            result: None,
        };
        fused.function.body.push(fused_call_stmt, Span::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! binding {
        ($g:literal, $b:literal) => {
            FusedResource::Binding {
                group: $g,
                binding: $b,
            }
        };
    }
    const sample_text_1: &str = "
	struct Output { field_0 : i32 }
	fn do_thing_on_gpu(a : i32) -> Output 
	{
		var output : Output;
		output.field_0 = a;
		return output;
	}
	
	struct Input_0 { field_0 : i32 }
	@group(0)  @binding(0) var<storage, read> input_0 : Input_0;
	struct Output_0 { field_0 : i32 }
	@group(0) @binding(1) var<storage, read_write> output_0 : Output_0;
	@compute @workgroup_size(1) fn main()
	{
		let output = do_thing_on_gpu(input_0.field_0);
		output_0.field_0 = output.field_0;
	}
	";

    #[test]
    fn test_naga_sanity() {
        let mut shader_module = ShaderModule::from_wgsl(sample_text_1).unwrap();
        let wgsl_text = shader_module.emit_wgsl();
    }

    #[test]
    fn test_fusion_sanity() {
        let pre = ShaderModule::from_wgsl(sample_text_1).unwrap();
        let mut map = HashMap::new();
        map.insert((0, 0, 0), binding!(0, 0));
        map.insert((0, 0, 1), binding!(0, 1));
        let fused = ShaderModule::fuse(FuseDescriptor {
            sources: &[FuseSource {
                shader_module: &pre,
                entry_point: "main",
            }],
            resources: &map,
        });
        let wgsl_text = fused.emit_wgsl();
        eprintln!("{wgsl_text}");
    }

    const fadd_wgpu: &str = "
    @group(0) @binding(0) var<storage, read> lhs : array<f32, 1024>;
    @group(0) @binding(1) var<storage, read> rhs : array<f32, 1024>;
    @group(0) @binding(2) var<storage, write> output : array<f32, 1024>;

    @compute @workgroup_size(256) 
    fn main(@builtin(local_invocation_index) i : u32)
    {
        output[i] = lhs[i] + rhs[i];
    }";

    #[test]
    fn test_fusion_duplicate() {
        let pre = ShaderModule::from_wgsl(fadd_wgpu).unwrap();

        let mut map = HashMap::new();
        map.insert((0, 0, 0), binding!(0, 0));
        map.insert((0, 0, 1), binding!(0, 1));
        map.insert((0, 0, 2), binding!(0, 2));
        // reuse output from the first as the second output to the second
        map.insert((1, 0, 0), binding!(0, 3));
        map.insert((1, 0, 1), binding!(0, 2));
        map.insert((1, 0, 2), binding!(0, 4));

        let fused = ShaderModule::fuse(FuseDescriptor {
            sources: &[
                FuseSource {
                    shader_module: &pre,
                    entry_point: "main",
                },
                FuseSource {
                    shader_module: &pre,
                    entry_point: "main",
                },
            ],
            resources: &map,
        });

        let wgsl_text = fused.emit_wgsl();
        eprintln!("{wgsl_text}");
    }

    const struct00: &str = "
    struct Struct00 { field_from_struct00 : f32 }
    @group(0) @binding(0) var<storage, write> struct00_out : Struct00;
    @compute @workgroup_size(1) fn main() { 
        struct00_out.field_from_struct00 = 7; 
    }";

    const struct01: &str = "
    struct Struct01 { field_from_struct01 : f32 }
    @group(0) @binding(0) var<storage, read> struct01_in : Struct01;
    @group(0) @binding(1) var<storage, write> struct01_out : Struct01;
    @compute @workgroup_size(1) fn not_named_main() { 
        struct01_out.field_from_struct01 = struct01_in.field_from_struct01; 
    }";

    #[test]
    fn test_fusion_structs() {
        let struct00_mod = ShaderModule::from_wgsl(struct00).unwrap();
        let struct01_mod = ShaderModule::from_wgsl(struct01).unwrap();

        let mut map = HashMap::new();
        map.insert((0, 0, 0), binding!(0, 0));
        map.insert((1, 0, 0), binding!(0, 0));
        map.insert((1, 0, 1), binding!(0, 1));

        let fused = ShaderModule::fuse(FuseDescriptor {
            sources: &[
                FuseSource {
                    shader_module: &struct00_mod,
                    entry_point: "main",
                },
                FuseSource {
                    shader_module: &struct01_mod,
                    entry_point: "not_named_main",
                },
            ],
            resources: &map,
        });

        let wgsl_text = fused.emit_wgsl();
        eprintln!("{wgsl_text}");
    }

    #[test]
    fn test_fusion_elide_bindings() {
        let struct00_mod = ShaderModule::from_wgsl(struct00).unwrap();
        let struct01_mod = ShaderModule::from_wgsl(struct01).unwrap();

        let mut map = HashMap::new();
        map.insert((0, 0, 0), FusedResource::Global(0));
        map.insert((1, 0, 0), FusedResource::Global(0));
        map.insert((1, 0, 1), binding!(0, 0));

        let fused = ShaderModule::fuse(FuseDescriptor {
            sources: &[
                FuseSource {
                    shader_module: &struct00_mod,
                    entry_point: "main",
                },
                FuseSource {
                    shader_module: &struct01_mod,
                    entry_point: "not_named_main",
                },
            ],
            resources: &map,
        });

        let wgsl_text = fused.emit_wgsl();
        eprintln!("{wgsl_text}");
    }
}
