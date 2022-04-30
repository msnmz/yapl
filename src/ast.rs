#[derive(Debug, Clone)]
pub enum Prog<'a> {
    Body(Vec<Stmt<'a>>),
}

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    ArithmeticAssign {
        name: &'a str,
        opcode: ArithmeticOpcode,
        rhs: Expr<'a>,
    },
    Assign {
        lhs: Expr<'a>,
        rhs: Expr<'a>,
    },
    Expr(Expr<'a>),
    IfElse {
        if_block: CondBlock<'a>,
        else_if_blocks: Vec<CondBlock<'a>>,
        else_block: Option<Vec<Stmt<'a>>>,
    },
    While(CondBlock<'a>),
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Ident(&'a str),
    Underscore,
    Int(i64),
    StrLiteral(&'a str),
    List(Vec<ListItem<'a>>),
    Call {
        func: &'a str,
        args: Vec<Expr<'a>>,
    },
    Op {
        lhs: Box<Expr<'a>>,
        rhs: Box<Expr<'a>>,
        opcode: Opcode,
    },
}

#[derive(Debug, Clone)]
pub struct ListItem<'a> {
    pub expr: Expr<'a>,
    pub is_spread: bool,
}

#[derive(Debug, Clone)]
pub struct CondBlock<'a> {
    pub cond: Expr<'a>,
    pub stmts: Vec<Stmt<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Copy)]
pub enum ArithmeticOpcode {
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}
