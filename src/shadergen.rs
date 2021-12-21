use crate::ir;

pub struct ShaderModule
{
	shader_module : naga::Module
}

impl ShaderModule
{
	fn new_with_naga_module(module : naga::Module) -> Self
	{
		Self{shader_module : module}
	}

	pub fn new_with_wgsl(text : &str) -> Self
	{
		match naga::front::wgsl::parse_str(text)
		{
			Err(why) => panic!("Error while parsing WGSL: {}", why),
			Ok(module) => Self::new_with_naga_module(module)
		}
	}
}


#[cfg(test)]
mod tests
{
	use crate::shadergen;
	use crate::ir;

	const sample_text_1 : &str = "
	struct Output {field_0 : i32;};
	fn do_thing_on_gpu(a : i32) -> Output 
	{
		var output : Output;
		output.field_0 = a;
		return output;
	}
	
	struct Input_0 { field_0 : i32; };
	[[group(0), binding(0)]] var<storage, read> input_0 : Input_0;
	struct Output_0 { field_0 : i32; };
	[[group(0), binding(1)]] var<storage, read_write> output_0 : Output_0;
	[[stage(compute), workgroup_size(1, 1, 1)]] fn main()
	{
		let output = do_thing_on_gpu(input_0.field_0);
		output_0.field_0 = output.field_0;
	}
	";

	#[test]
	fn test_naga_sanity()
	{
		let mut shader_module = shadergen::ShaderModule::new_with_wgsl(sample_text_1);
		//shader_generator.compile();
	}
}