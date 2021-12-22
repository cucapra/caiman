use std::fmt::Write;
use std::default::Default;

#[derive(Default)]
struct TextSection
{
	code_string : String
}

/*enum WritingState
{
	Root{section_id : usize},
	Pipeline{section_id : usize, output_section_id : usize },
}*/

pub struct CodeWriter
{
	//code_string : String,
	sections : Vec<TextSection>,
	//writing_states : Vec<WritingState>,
	active_section : usize,
}

impl CodeWriter
{
	pub fn new() -> Self
	{
		let sections = vec![TextSection{code_string : String::new()}];
		//let writing_states = vec![WritingState::Root{section_id : 0}];
		Self { sections, /*writing_states,*/ active_section : 0 }
	}

	pub fn finish(&mut self) -> String
	{
		let mut code_string = String::new();
		for section in self.sections.iter()
		{
			code_string += &section.code_string;
		}
		code_string
	}

	fn create_section(&mut self) -> usize
	{
		let id = self.sections.len();
		self.sections.push(TextSection{code_string : String::new()});
		id
	}

	fn set_active_section(&mut self, to : usize)
	{
		self.active_section = to;
	}

	fn break_section(&mut self) -> usize
	{
		let section_id = self.create_section();
		self.set_active_section(section_id);
		section_id
	}

	/*fn push_writing_state(&mut self, state : WritingState)
	{
		self.writing_states.push(state);
	}

	fn pop_writing_state(&mut self)
	{
		self.writing_states.pop();
	}*/
	
	fn get_active_section_ptr(&mut self) -> &mut TextSection
	{
		&mut self.sections[self.active_section]
	}

	pub fn begin_pipeline(&mut self, pipeline_name : &str)
	{
		write!(self.get_active_section_ptr().code_string, "pub mod {} {{\n", pipeline_name);
		self.break_section();
	}

	pub fn end_pipeline(&mut self)
	{
		self.get_active_section_ptr().code_string += "}\n";
	}

	/*fn write_line(&mut self, line : &String)
	{

	}*/

	pub fn write(&mut self, text : String)
	{
		self.get_active_section_ptr().code_string += text.as_str();
	}
}

impl Write for CodeWriter
{
	fn write_str(&mut self, text : &str) -> Result<(), std::fmt::Error>
	{
		self.get_active_section_ptr().code_string.write_str(text)
	}
}

/*pub struct PipelineWriter
{
	code_writer : CodeWriter,
	section
}

impl PipelineWriter
{
	fn new(code_writer : CodeWriter) -> Self
	{
		Self{code_writer}
	}
}

impl Write for PipelineWriter
{
	fn write_str(&mut self, text : &str) -> Result<(), std::fmt::Error>
	{
		self.sections[].code_string.write_str(text)
	}
}*/

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
