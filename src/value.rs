use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Unit,
    Int(i64),
    Boolean(bool),
    StrLiteral(&'a str),
    List(Vec<Value<'a>>),
    Func(fn(Vec<Value<'a>>) -> Result<Value<'a>, String>),
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        if discriminant(self) != discriminant(other) {
            return false;
        }
        match (self, other) {
            (&Value::Boolean(s), &Value::Boolean(o)) => s == o,
            (&Value::Int(s), &Value::Int(o)) => s == o,
            (&Value::StrLiteral(s), &Value::StrLiteral(o)) => s == o,
            (&Value::Unit, &Value::Unit) => true,
            _ => false,
        }
    }
}

impl<'a> PartialOrd for Value<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::mem::discriminant;
        if discriminant(self) != discriminant(other) {
            return None;
        }
        match (self, other) {
            (&Value::Boolean(s), &Value::Boolean(o)) => s.partial_cmp(&o),
            (&Value::Int(s), &Value::Int(o)) => s.partial_cmp(&o),
            (&Value::StrLiteral(s), &Value::StrLiteral(o)) => s.partial_cmp(&o),
            _ => None,
        }
    }
}

macro_rules! impl_value {
    ($_trait:ty > $_fn:ident for $_self:ty => $_out_type:ty as $op:tt) => {
        impl<'a, 'b> $_trait for &'b Value<'a> {
            type Output = $_out_type;
            fn $_fn(self, rhs: $_self) -> Self::Output {
                use std::mem::discriminant;
                if discriminant(self) != discriminant(rhs) {
                    return Err(format!("type mismatch: '{}' operation not permitted on these types", stringify!($op)));
                }
                match (self, &rhs) {
                    (Value::Int (l), &Value::Int (r)) => Ok(Value::Int (l $op r)),
                    _ => Err(format!("impossible op: '{}' operation not permitted on these types", stringify!($op))),
                }
            }
        }
    };
}

impl_value!(Add<&'b Value<'a>> > add for Self => Result<Value<'a>, String> as +);
impl_value!(Sub<&'b Value<'a>> > sub for Self => Result<Value<'a>, String> as -);
impl_value!(Div<&'b Value<'a>> > div for Self => Result<Value<'a>, String> as /);
impl_value!(Mul<&'b Value<'a>> > mul for Self => Result<Value<'a>, String> as *);
impl_value!(Rem<&'b Value<'a>> > rem for Self => Result<Value<'a>, String> as %);
