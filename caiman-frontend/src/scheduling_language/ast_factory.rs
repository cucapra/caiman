use crate::scheduling_language::ast::*;

pub struct ASTFactory 
{ 
    line_ending_byte_offsets: Vec<usize>,
}

impl ASTFactory
{
    
    pub fn new(s: &str) -> Self { 
        Self {
            line_ending_byte_offsets: s
                .as_bytes()
                .iter()
                .enumerate()
                .filter_map(
                    |(idx, b)| if *b == b'\n' { Some(idx) } else { None }
                )
                .collect(),
        } 
    }

    pub fn line_and_column(&self, u: usize) -> (usize, usize)
    {
        if let Some(b) = self.line_ending_byte_offsets.last() 
        {
            if u > *b { panic!("Byte offset too big: {}", u); }
        }
        self.line_ending_byte_offsets
            .iter()
            .enumerate()
            .map(|(l, c)| (l + 1, c))
            .fold(
                (1, u), // Case where offset is on line one
                |curr, (l, c)| if u > *c { (l + 1, u - c) } else { curr },
            )
    }

    fn info(&self, l: usize, r: usize) -> Info
    {
        Info {
            location: (self.line_and_column(l), self.line_and_column(r)),
        }
    }

    pub fn var(&self, l: usize, v: String, r: usize) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::Var(v))
    }
}
