use std::collections::HashMap;
use crate::value_language::check::SemanticError;

type Var = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    Any,
}

pub type FuncType = (Vec<Type>, Type);

#[derive(Clone)]
pub struct Context 
{ 
    // Inner bool is true if mutation is allowed on 
    // that variable
    var_table: HashMap<Var, (Type, bool)>,
}

#[derive(Clone)]
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

    pub fn get(&self, f: Var) -> Result<(Vec<Type>, Type), SemanticError>
    {
        match self.table.get(&f)
        {
            Some((v, rt)) => Ok((v.to_vec(), *rt)),
            None => Err(SemanticError::UnboundFunction(f.to_string())),
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
        use ExprType::*;
        match self
        {
            Ordinary(ord_t) => ord_t.is_subtype_of(t),
            PositiveInteger => t == Type::I32,
            NegativeInteger => t == Type::I32,
            Float => panic!("Float moment"),
            Any => true,
        }
    }

    // Bleh
    pub fn compatible(&self, et: ExprType) -> bool
    {
        use ExprType::*;
        match (self, et)
        {
            (Ordinary(t1), Ordinary(t2)) => *t1 == t2,
            (Ordinary(t1), _) => et.is_subtype_of(*t1),
            (_, Ordinary(t2)) => self.is_subtype_of(t2),
            (NegativeInteger, NegativeInteger)
            | (PositiveInteger, PositiveInteger)
            | (PositiveInteger, NegativeInteger)
            | (NegativeInteger, PositiveInteger)
            | (Float, Float) 
            | (Any, _) 
            | (_, Any) => true,
            _ => false,
        }
    }

    fn order(&self) -> usize
    {
        use ExprType::*;
        match self
        {
            Ordinary(_) => 0,
            Float => 1,
            NegativeInteger => 2,
            PositiveInteger => 3,
            Any => 4,
        }
    }

    pub fn is_number(&self) -> bool
    {
        use ExprType::*;
        match self
        {
            | PositiveInteger 
            | NegativeInteger 
            | Any
            | Float => true,
            Ordinary(t) => match *t {
                Type::I32 => true,
                Type::Bool => false,
            },
        }
    }

    pub fn is_bool(&self) -> bool
    {
        self.is_subtype_of(Type::Bool)
    }
}

pub fn type_equal(t1: Type, t2: Type) -> bool
{
    t1 == t2
}

pub fn min_expr_type(t1: ExprType, t2: ExprType) -> ExprType
{
    if t1.order() >= t2.order() { t1 } else { t2 }
}

