
pub struct CodeWriter
{
	code_string : String
}

impl CodeWriter
{
	pub fn new() -> Self
	{
		Self { code_string : String::new() }
	}

	pub fn finish(&mut self) -> String
	{
		self.code_string.clone()
	}

	/*fn begin_pipeline(&mut self, name : &String)
	{

	}

	fn end_pipeline(&mut self)
	{

	}*/

	/*fn write_line(&mut self, line : &String)
	{

	}*/

	pub fn write(&mut self, text : String)
	{
		self.code_string += text.as_str();
	}

	pub fn write_str(&mut self, text : &str)
	{
		self.code_string += text;
	}
}

pub struct VariableTracker
{
	next_id : usize
}

impl VariableTracker
{
	pub fn new() -> Self
	{
		Self { next_id : 0 }
	}

	pub fn generate(&mut self) -> usize
	{
		let id = self.next_id;
		self.next_id += 1;
		id
	}
}
