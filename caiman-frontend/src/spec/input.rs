use caiman::ir;

pub enum SpecNodeInput
{
    Usize(usize),
    I64(i64),
    U64(u64),
    I32(i32),
    UsizeSlice(Box<[usize]>),
}

impl Default for SpecNodeInput
{
    fn default() -> Self {
        SpecNodeInput::Usize(0)
    }
}

macro_rules! unwrapper 
{
    ($f:ident, $rt:ty, $p:pat, $p_extract:expr, $strname:expr) => {
        pub fn $f (self) -> $rt
        {
            match self
            {
                $p => $p_extract,
                _ => panic!("Expected {} as input to spec node", $strname),
            }
        }
    }
}

impl SpecNodeInput
{
    unwrapper!(unwrap_u64, u64, SpecNodeInput::U64(u), u, "u64");
    unwrapper!(unwrap_usize_slice, Box<[usize]>, SpecNodeInput::UsizeSlice(u), u, "Box<[usize]>");
    unwrapper!(unwrap_i64, i64, SpecNodeInput::I64(i), i, "i64");
    unwrapper!(unwrap_i32, i32, SpecNodeInput::I32(i), i, "i32");
    unwrapper!(unwrap_usize, usize, SpecNodeInput::Usize(u), u, "usize");

    pub fn unwrap_irconstant(self) -> ir::Constant
    {
        match self
        {
            SpecNodeInput::I64(i) => ir::Constant::I64(i),
            SpecNodeInput::U64(u) => ir::Constant::U64(u),
            SpecNodeInput::I32(i) => ir::Constant::I32(i),
            _ => panic!("Expected IR constant as input to spec node"),
        }
    }
}
