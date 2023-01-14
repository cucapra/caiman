pub enum SpecNodeInput
{
    Usize(usize),
    I64(i64),
    U64(u64),
    UsizeSlice(Box<[usize]>),
}

impl Default for SpecNodeInput
{
    fn default() -> Self {
        SpecNodeInput::Usize(0)
    }
}

impl SpecNodeInput
{
    pub fn unwrap_u64(self) -> u64
    {
        match self
        {
            SpecNodeInput::U64(u) => u,
            _ => panic!("Expected u64 as input to spec node"),
        }
    }

    pub fn unwrap_usize_slice(self) -> Box<[usize]>
    {
        match self
        {
            SpecNodeInput::UsizeSlice(u) => u,
            _ => panic!("Expected Box<[usize]> as input to spec node"),
        }
    }

    pub fn unwrap_i64(self) -> i64
    {
        match self
        {
            SpecNodeInput::I64(i) => i,
            _ => panic!("Expected i64 as input to spec node"),
        }
    }

    pub fn unwrap_usize(self) -> usize
    {
        match self
        {
            SpecNodeInput::Usize(u) => u,
            _ => panic!("Expected usize as input to spec node"),
        }
    }
}
