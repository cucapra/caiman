use crate::ir;
use std::error::Error;

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
