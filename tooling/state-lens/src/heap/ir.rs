use crate::model::Span;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lang {
    JavaScript,
    Python,
    Go,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Binding(Span),
    Builtin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequirementKind {
    DeclaredReadonly,
    Frozen,
    ReadOnlyParameter,
}

impl RequirementKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::DeclaredReadonly => "declared-readonly",
            Self::Frozen => "frozen",
            Self::ReadOnlyParameter => "read-only-parameter",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub span: Span,
    pub default: Option<Expr>,
    pub requirement: Option<(RequirementKind, Span)>,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub span: Span,
    pub params: Vec<Param>,
    pub receiver: Option<Param>,
    pub body: Vec<Stmt>,
    pub expression_body: Option<Expr>,
    pub defaults_at_definition: bool,
    pub receiver_by_value: bool,
    pub opaque: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub enum Index {
    Constant(u64),
    Name(String),
    Dynamic(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Entry {
    Field(String, Expr),
    Element(Expr),
    Spread(Expr),
    Opaque(&'static str, Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Container {
    Object,
    Array,
    Map,
    Struct,
    Set,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var {
        name: String,
        span: Span,
        key: Key,
    },
    Field {
        base: Box<Expr>,
        name: String,
        span: Span,
    },
    Index {
        base: Box<Expr>,
        index: Index,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Literal {
        container: Container,
        type_name: Option<String>,
        entries: Vec<Entry>,
        elements: Vec<Expr>,
        span: Span,
        value_semantics: bool,
    },
    NewArray {
        length: Option<Box<Expr>>,
        span: Span,
    },
    Repeat {
        element: Box<Expr>,
        count: Box<Expr>,
        span: Span,
    },
    Comprehension {
        element: Box<Expr>,
        count: Option<u64>,
        span: Span,
    },
    Slice {
        base: Box<Expr>,
        low: Option<Box<Expr>>,
        high: Option<Box<Expr>>,
        max: Option<Box<Expr>>,
        span: Span,
    },
    AddressOf {
        operand: Box<Expr>,
        span: Span,
    },
    Function(Rc<Func>),
    Integer(i64, Span),
    Null(Span),
    Choice(Vec<Expr>, Span),
    Sequence(Vec<Expr>, Span),
    Assign(Box<Stmt>, Span),
    Scalar(Vec<Expr>, Span),
    Variadic(Box<Expr>, Span),
    Named(String, Box<Expr>, Span),
    Unknown {
        reason: &'static str,
        inputs: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Var { span, .. }
            | Self::Field { span, .. }
            | Self::Index { span, .. }
            | Self::Call { span, .. }
            | Self::Literal { span, .. }
            | Self::NewArray { span, .. }
            | Self::Repeat { span, .. }
            | Self::Comprehension { span, .. }
            | Self::Slice { span, .. }
            | Self::AddressOf { span, .. }
            | Self::Integer(_, span)
            | Self::Null(span)
            | Self::Choice(_, span)
            | Self::Sequence(_, span)
            | Self::Assign(_, span)
            | Self::Scalar(_, span)
            | Self::Variadic(_, span)
            | Self::Named(_, _, span)
            | Self::Unknown { span, .. } => *span,
            Self::Function(func) => func.span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Bind {
        key: Key,
        name: String,
        span: Span,
        statement: Span,
        value: Option<Expr>,
        requirement: Option<(RequirementKind, Span)>,
    },
    Assign {
        target: Expr,
        value: Expr,
        span: Span,
    },
    Eval(Expr),
    If {
        condition: Expr,
        then: Vec<Stmt>,
        otherwise: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Throw {
        value: Option<Expr>,
        span: Span,
    },
    Loop {
        header: Vec<Stmt>,
        body: Vec<Stmt>,
        span: Span,
    },
    Try {
        body: Vec<Stmt>,
        handler: Vec<Stmt>,
        finally: Vec<Stmt>,
        span: Span,
    },
    Method {
        type_name: String,
        name: String,
        func: Rc<Func>,
    },
    Unsupported {
        reason: &'static str,
        span: Span,
        binds: Vec<(Key, String, Span)>,
    },
}
