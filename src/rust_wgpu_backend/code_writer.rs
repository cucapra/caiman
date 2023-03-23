use std::default::Default;
use std::fmt::Write;

#[derive(Default)]
struct TextSection {
    code_string: String,
}

pub struct CodeWriter {
    //code_string : String,
    sections: Vec<TextSection>,
    //writing_states : Vec<WritingState>,
    active_section: usize,
}

impl CodeWriter {
    pub fn new() -> Self {
        let sections = vec![TextSection {
            code_string: String::new(),
        }];
        //let writing_states = vec![WritingState::Root{section_id : 0}];
        Self {
            sections,
            /*writing_states,*/ active_section: 0,
        }
    }

    pub fn finish(&mut self) -> String {
        let mut code_string = String::new();
        for section in self.sections.iter() {
            code_string += &section.code_string;
        }
        code_string
    }

    fn create_section(&mut self) -> usize {
        let id = self.sections.len();
        self.sections.push(TextSection {
            code_string: String::new(),
        });
        id
    }

    fn set_active_section(&mut self, to: usize) {
        self.active_section = to;
    }

    fn break_section(&mut self) -> usize {
        let section_id = self.create_section();
        self.set_active_section(section_id);
        section_id
    }

    pub fn begin_module(&mut self, name: &str) {
        write!(self, "pub mod {} {{\n", name);
    }

    pub fn end_module(&mut self) {
        write!(self, "}}\n");
    }

    pub fn begin_struct(&mut self, name: &str) {
        write!(self, "pub struct {} {{", name);
    }

    pub fn write_struct_field(&mut self, index: usize, type_name: &str) {
        self.write(format!("pub field_{} : {}, ", index, type_name));
    }

    pub fn end_struct(&mut self) {
        write!(self, "}}\n");
    }

    fn get_active_section_ptr(&mut self) -> &mut TextSection {
        &mut self.sections[self.active_section]
    }

    pub fn write(&mut self, text: String) {
        self.get_active_section_ptr().code_string += text.as_str();
    }
}

impl Write for CodeWriter {
    fn write_str(&mut self, text: &str) -> Result<(), std::fmt::Error> {
        self.get_active_section_ptr().code_string.write_str(text)
    }
}
