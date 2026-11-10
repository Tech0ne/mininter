#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Isa,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
    New,
    AddressOf, // @
}

/// Function parameter with optional default value.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    StringLit(String),
    Bool(bool),
    Null,
    Identifier(String),
    Self_,
    GlobalAccess(String), // globals.field

    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    /// Function call with parentheses or paren-free: `f(x)` or `f x`
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    /// Index access: `a[i]`
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// Slice access: `a[i:j]`  (either bound can be None)
    Slice {
        object: Box<Expr>,
        from: Option<Box<Expr>>,
        to: Option<Box<Expr>>,
    },

    /// Member access: `a.b`
    Member {
        object: Box<Expr>,
        field: String,
    },

    /// List literal: `[1, 2, 3]`
    ListLiteral(Vec<Expr>),

    /// Map literal: `{"a": 1, "b": 2}`
    MapLiteral(Vec<(Expr, Expr)>),

    /// Anonymous function: `function(x, y) ... end function`
    FuncExpr {
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
}

/// Assignment targets for `=` and compound assignment.
#[derive(Debug, Clone)]
pub enum AssignTarget {
    Variable(String),
    Index { object: Box<Expr>, index: Box<Expr> },
    Member { object: Box<Expr>, field: String },
    Global(String), // globals.field = ...
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Assignment {
        target: AssignTarget,
        value: Expr,
    },

    If {
        /// (condition, body) pairs for if / else if branches
        branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },

    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    Module(String),
}
