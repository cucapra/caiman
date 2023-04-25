use crate::ir;
use std::collections::HashSet;
use std::error::Error;

#[derive(Debug)]
pub struct FuseDescriptor<'a> {
    /// The modules to fuse into a single kernel dispatch, ordered by execution.
    modules: &'a [ShaderModule],
    /// A list of mergers. Each merger specifies multiple global variables which should be merged
    /// into the same resulting binding. The global variables are identified by means of a
    /// (index into modules list, group number, binding number) tuple.
    mergers: Vec<HashSet<(usize, u32, u32)>>,
}

#[derive(Debug)]
pub struct ShaderModule {
    shader_module: naga::Module,
}

impl ShaderModule {
    pub fn from_wgsl(text: &str) -> Result<Self, Box<dyn Error>> {
        let shader_module = naga::front::wgsl::parse_str(text)?;
        Ok(Self { shader_module })
    }
    pub fn from_glsl(text: &str) -> Result<Self, Box<dyn Error>> {
        let mut parser = naga::front::glsl::Frontend::default();
        let options = naga::front::glsl::Options::from(naga::ShaderStage::Compute);
        match parser.parse(&options, text) {
            Ok(shader_module) => Ok(Self { shader_module }),
            Err(errors) => {
                // Just report the first error for now.
                // Could do something better here in the future, but there's a lot of
                // error handling work that needs to be done beforehand...
                Err(Box::new(errors.into_iter().next().unwrap()))
            }
        }
    }
    pub fn emit_wgsl(&mut self) -> String {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        );
        let module_info = match validator.validate(&self.shader_module) {
            Err(why) => panic!("Error while validating WGSL: {}", why),
            Ok(module_info) => module_info,
        };
        match naga::back::wgsl::write_string(
            &self.shader_module,
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
        let mut global_variables = naga::Arena::new();
        let mut existing_mergers = vec![None; desc.mergers.len()];

        for (module_index, module) in desc.modules.iter().enumerate() {
            let module = &module.shader_module;
            let type_map = naga_fuse::fuse_types(module, &mut types);
            let cst_map = naga_fuse::fuse_constants(module, &type_map, &mut constants);
            let gv_map = naga_fuse::fuse_global_variables(
                module,
                module_index,
                &type_map,
                &cst_map,
                &desc.mergers[..],
                &mut existing_mergers[..],
                &mut global_variables,
            );
        }
        todo!();
    }
}

mod naga_fuse {
    use naga::{
        Arena, Constant, ConstantInner, GlobalVariable, Handle, Module, ResourceBinding, Type,
        UniqueArena,
    };
    use std::collections::{HashMap, HashSet};

    use super::FuseDescriptor;

    type Remap<T> = HashMap<Handle<T>, Handle<T>>;

    pub fn fuse_types(module: &Module, types: &mut UniqueArena<Type>) -> Remap<Type> {
        // maps original type handles to merged type handles
        let mut type_map = HashMap::new();
        for (old_handle, ty) in module.types.iter() {
            let span = module.types.get_span(old_handle);
            let new_handle = types.insert(ty.clone(), span);
            type_map.insert(old_handle, new_handle);
        }
        // we shouldn't be raytracing in a compute shader
        assert!(module.special_types.ray_desc.is_none());
        assert!(module.special_types.ray_intersection.is_none());
        return type_map;
    }

    pub fn fuse_constants(
        module: &Module,
        type_map: &Remap<Type>,
        constants: &mut Arena<Constant>,
    ) -> Remap<Constant> {
        let mut cst_map = HashMap::new();
        for (old_handle, old_cst) in module.constants.iter() {
            let span = module.constants.get_span(old_handle);
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
                name: old_cst.name.clone(),
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

    pub fn fuse_resource_binding() {}
    pub fn fuse_global_variables(
        module: &Module,
        module_index: usize,
        type_map: &Remap<Type>,
        cst_map: &Remap<Constant>,
        mergers: &[HashSet<(usize, u32, u32)>],
        existing_mergers: &mut [Option<Handle<ResourceBinding>>],
        global_variables: &mut Arena<GlobalVariable>,
    ) -> Remap<GlobalVariable> {
        let mut gv_map = HashMap::new();
        for (old_handle, old_gv) in module.global_variables.iter() {
            let span = module.global_variables.get_span(old_handle);
            if let Some(ResourceBinding { group, binding }) = &old_gv.binding {
                // This is a resource binding, which means we need to merge it.
            } else {
                // This is just a normal global variable.
                // Remap fields & insert it
                let new_gv = GlobalVariable {
                    name: old_gv.name.clone(),
                    space: old_gv.space.clone(),
                    binding: None,
                    ty: type_map[&old_gv.ty],
                    init: old_gv.init.map(|old_cst| cst_map[&old_cst]),
                };
                let new_handle = global_variables.append(new_gv, span);
                gv_map.insert(old_handle, new_handle);
            }
        }
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::ir;
    use crate::shadergen;

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
        let mut shader_module = shadergen::ShaderModule::from_wgsl(sample_text_1).unwrap();
        let wgsl_text = shader_module.emit_wgsl();
    }
}
