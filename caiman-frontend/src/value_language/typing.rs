use std::collections::HashMap;
use crate::value_language::check::SemanticError;

type Var = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type
{
    I32,
    //F32,
    Bool,
}

// TODO: Display for expression types
// (and regular type too I guess)

// Differs from Type in that it is inferred (e.g. it is the
// type of the number 1)
// Size will be checked later
#[derive(Clone, Copy, Debug)]
pub enum ExprType
{
    Ordinary(Type),
    PositiveInteger,
    NegativeInteger,
    Float,
}

pub type FuncType = (Vec<Type>, Type);

#[derive(Clone)]
pub struct Context 
{ 
    // Inner bool is true if mutation is allowed on 
    // that variable
    var_table: HashMap<Var, (Type, bool)>,
}

pub struct FunctionContext 
{
    table: HashMap<Var, FuncType>,
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

    fn get(&self, v: &Var) -> Result<(Type, bool), SemanticError>
    {
        match self.var_table.get(v)
        {
            Some(p) => Ok(*p),
            None => Err(SemanticError::UnboundVariable(v.to_string())),
        }
    }

    pub fn var_is_mutable(&self, v: &Var) -> Result<bool, SemanticError>
    {
        let (_, is_mut) = self.get(v)?;
        Ok(is_mut)
    }
    
    pub fn add(
        &mut self, 
        v: Var, 
        t: Type, 
        m: bool,
    ) -> Result<(), SemanticError>
    {
        // Shadowing allowed.
        self.var_table.insert(v, (t, m));
        Ok(())
    }

    pub fn get_type(&self, v: &Var) -> Result<Type, SemanticError>
    {
        let (t, _) = self.get(&v)?;
        Ok(t)
    }
}

impl FunctionContext
{
    pub fn new() -> Self 
    {
        Self { table: HashMap::new() }
    }

    pub fn add(
        &mut self, 
        v: Var, 
        param_ts: Vec<Type>, 
        ret: Type,
    ) -> Result<(), SemanticError>
    {
        match self.table.insert(v.clone(), (param_ts, ret))
        {
            Some(_) => Err(SemanticError::FunctionNameCollision(v)),
            None => Ok(())
        }
    }
}

impl Type
{
    pub fn is_subtype_of(&self, t: Type) -> bool
    {
        match self
        {
            Type::I32 => t == Type::I32,
            Type::Bool => t == Type::Bool,
        }
    }
}

impl ExprType
{
    pub fn is_subtype_of(&self, t: Type) -> bool
    {
        match self
        {
            ExprType::Ordinary(ord_t) => ord_t.is_subtype_of(t),
            ExprType::PositiveInteger => t == Type::I32,
            ExprType::NegativeInteger => t == Type::I32,
            ExprType::Float => panic!("Float moment"),
        }
    }

    // Bleh
    pub fn compatible(&self, et: ExprType) -> bool
    {
        match (self, et)
        {
            (ExprType::Ordinary(t1), ExprType::Ordinary(t2)) => *t1 == t2,
            (ExprType::Ordinary(t1), _) => et.is_subtype_of(*t1),
            (_, ExprType::Ordinary(t2)) => self.is_subtype_of(t2),
            (ExprType::NegativeInteger, ExprType::NegativeInteger)
            | (ExprType::PositiveInteger, ExprType::PositiveInteger)
            | (ExprType::PositiveInteger, ExprType::NegativeInteger)
            | (ExprType::NegativeInteger, ExprType::PositiveInteger)
            | (ExprType::Float, ExprType::Float) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool
    {
        match self
        {
            | ExprType::PositiveInteger 
            | ExprType::NegativeInteger 
            | ExprType::Float => true,
            ExprType::Ordinary(t) => match *t {
                Type::I32 => true,
                Type::Bool => false,
            },
        }
    }

    pub fn is_bool(&self) -> bool
    {
        match self
        {
            ExprType::Ordinary(t) => *t == Type::Bool,
            _ => false,
        }
    }
}

pub fn type_equal(t1: Type, t2: Type) -> bool
{
    t1 == t2
}


