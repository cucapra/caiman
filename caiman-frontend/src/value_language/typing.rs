use std::collections::HashMap;

type Var = String;

#[derive(Clone, Copy)]
pub enum Type
{
    I32,
}

#[derive(Clone)]
pub struct Context 
{
    // Inner bool is true if mutation is allowed on 
    // that variable
    var_table: HashMap<Var, (Type, bool)>,
}

impl Context
{
    pub fn new() -> Self 
    {
        Context {
            var_table: HashMap::new(),
        }
    }

    pub fn contains_var(&self, v: Var) -> bool
    {
        self.var_table.contains_key(&v)
    }

    pub fn var_is_mutable(&self, v: Var) -> Option<bool>
    {
        self.var_table.get(&v).map(|(_, is_mut)| *is_mut)
    }
    
    pub fn add(&mut self, v: Var, t: Type, m: bool)
    {
        self.var_table.insert(v, (t, m));
    }
}

