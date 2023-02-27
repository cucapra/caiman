// The "Value Intermediate Language" is an intermediate representation
// of the value language whose statements' subexpressions are broken up into
// individualized, labelable pieces. The intention behind this is to make it very
// easily compatible with the scheduling language.


pub struct Stmt 
{

}

pub type Program = Vec<Stmt>;
