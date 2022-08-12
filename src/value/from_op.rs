use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FromOpError {
    #[error("malformed operation")]
    Malformed,
    #[error("unknown operation kind {0}")]
    UnknownOp(String),
    #[error("unknown attribute {0}")]
    UnknownAttrib(String),
    #[error("invalid attribute {0}")]
    InvalidAttrib(String),
    #[error("missing attribute {0}")]
    MissingAttrib(String),
    #[error("duplicate attribute {0}")]
    DuplicateAttrib(String),
    #[error("wrong number of dependencies: expected {expected}, found {found}")]
    InvalidDeps { expected: usize, found: usize },
}
enum Token<'a> {
    Lit(&'a str),
    AttrStart, // = {
    AttrAssign,
    AttrDelim, // = ,
    AttrEnd,   // = }
}
fn tokenize(op: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut lit_start = 0;
    for (i, c) in op.char_indices() {
        let token = match c {
            '{' => Token::AttrStart,
            '=' => Token::AttrAssign,
            ',' => Token::AttrDelim,
            '}' => Token::AttrEnd,
            _ => continue,
        };
        if lit_start < i {
            tokens.push(Token::Lit(&op[lit_start..i]))
        }
        tokens.push(token);
        lit_start = i + 1;
    }
    tokens
}

pub struct Attribs<'a> {
    inner: HashMap<&'a str, (&'a str, bool)>,
}
impl<'a> Attribs<'a> {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    fn add(&mut self, k: &'a str, v: &'a str) -> Result<(), FromOpError> {
        match self.inner.insert(k, (v, false)) {
            Some((replaced, _used)) => Err(FromOpError::DuplicateAttrib(replaced.to_string())),
            None => Ok(()),
        }
    }
    pub fn get<T: FromStr>(&mut self, k: &str) -> Result<T, FromOpError> {
        match self.inner.get_mut(k) {
            None => return Err(FromOpError::MissingAttrib(k.into())),
            Some(entry) => match entry.0.parse() {
                Err(_) => Err(FromOpError::InvalidAttrib(entry.0.into())),
                Ok(parsed) => {
                    entry.1 = true;
                    Ok(parsed)
                }
            },
        }
    }
    pub fn unused(&self) -> impl '_ + Iterator<Item = &'a str> {
        self.inner
            .values()
            .filter_map(|(v, used)| (!used).then(|| *v))
    }
}
impl egg::FromOp for super::Node {
    type Error = FromOpError;
    fn from_op(op: &str, children: Vec<egg::Id>) -> Result<Self, Self::Error> {
        let tokens = tokenize(op);
        let (kind, mut attribs) = match tokens.as_slice() {
            [Token::Lit(kind)] => (*kind, Attribs::new()),
            [Token::Lit(kind), Token::AttrStart, attrs @ .., Token::AttrEnd] => {
                let mut map = Attribs::new();
                let mut rest = attrs;
                while let [Token::Lit(k), Token::AttrAssign, Token::Lit(v), tail @ ..] = rest {
                    map.add(*k, *v)?;
                    if let [Token::AttrDelim, tail_attrs @ ..] = tail {
                        rest = tail_attrs;
                    } else {
                        rest = tail;
                        break;
                    }
                }
                if !rest.is_empty() {
                    return Err(Self::Error::Malformed);
                }
                (*kind, map)
            }
            _ => return Err(Self::Error::Malformed),
        };

        let computed_kind = if kind == "id_list" {
            super::NodeKind::IdList
        } else if kind == "param" {
            let funclet_id = attribs.get("funclet_id")?;
            let index = attribs.get("index")?;
            if !children.is_empty() {
                return Err(Self::Error::InvalidDeps {
                    expected: 0,
                    found: children.len(),
                });
            }
            super::NodeKind::Param { funclet_id, index }
        } else {
            let op_kind = super::OperationKind::from_description(kind, &mut attribs)?;
            if op_kind.num_deps() != children.len() {
                return Err(Self::Error::InvalidDeps {
                    expected: op_kind.num_deps(),
                    found: children.len(),
                });
            }
            super::NodeKind::Operation { kind: op_kind }
        };

        if let Some(unused) = attribs.unused().next() {
            return Err(Self::Error::UnknownAttrib(unused.into()));
        }
        Ok(Self {
            kind: computed_kind,
            deps: children.into(),
        })
    }
}
