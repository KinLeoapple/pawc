// src/ast.rs
// AST definitions for PawScript

/// Trait for AST nodes carrying location information
pub trait AstNode {
    fn line(&self) -> usize;
    fn col(&self) -> usize;
}

/// Identifier with name and source position
#[derive(Debug, Clone)]
pub struct IdentifierNode<'a> {
    pub name: &'a str,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for IdentifierNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Module path: sequence of identifiers with position at start
#[derive(Debug, Clone)]
pub struct ModulePath<'a> {
    pub segments: Vec<IdentifierNode<'a>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for ModulePath<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Import declaration node
#[derive(Debug, Clone)]
pub struct ImportNode<'a> {
    pub path: ModulePath<'a>,
    pub alias: Option<IdentifierNode<'a>>,
}

#[derive(Clone, Debug)]
pub struct RecordInitFieldNode<'a> {
    pub name: IdentifierNode<'a>,
    pub expr: ExpressionNode<'a>,
    pub line: usize,
    pub col: usize,
}

#[derive(Clone, Debug)]
pub struct RecordInitNode<'a> {
    pub typename: IdentifierNode<'a>,
    pub fields: Vec<RecordInitFieldNode<'a>>,
    pub line: usize,
    pub col: usize,
}

/// Top-level declaration items
#[derive(Debug, Clone)]
pub enum TopLevelKind<'a> {
    ModuleImport(ImportNode<'a>),
    Function(FunctionDefinitionNode<'a>),
    Record(RecordDefinitionNode<'a>),
    Protocol(ProtocolDefinitionNode<'a>),
    Statement(StatementNode<'a>),
}

/// Wrapper for top-level items including location
#[derive(Debug, Clone)]
pub struct TopLevelItem<'a> {
    pub node: TopLevelKind<'a>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for TopLevelItem<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Core type names: simple or generic
#[derive(Debug, Clone)]
pub enum CoreTypeNameNode<'a> {
    Simple(IdentifierNode<'a>),
    Generic {
        name: IdentifierNode<'a>,
        type_args: Vec<TypeNameNode<'a>>,
    },
}

/// Type name node with optional marker
#[derive(Debug, Clone)]
pub struct TypeNameNode<'a> {
    pub core: CoreTypeNameNode<'a>,
    pub is_optional: bool,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for TypeNameNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Literal variants
#[derive(Debug, Clone)]
pub enum LiteralNode<'a> {
    Int(i64),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Char(char),
    StringLiteral(StringInterpolationNode<'a>),
    Nopaw,
}

/// String interpolation node: parts include literals or expressions
#[derive(Debug, Clone)]
pub struct StringInterpolationNode<'a> {
    pub parts: Vec<StringPartNode<'a>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for StringInterpolationNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Parts of a string: literal text or embedded expression
#[derive(Debug, Clone)]
pub enum StringPartNode<'a> {
    Text(&'a str),
    Expr(ExpressionNode<'a>),
}

/// Expression AST
#[derive(Debug, Clone)]
pub enum ExpressionNode<'a> {
    Literal(LiteralNode<'a>),
    ArrayLiteral(Vec<ExpressionNode<'a>>),
    BinaryOp {
        left: Box<ExpressionNode<'a>>,
        op: BinaryOp,
        right: Box<ExpressionNode<'a>>,
        line: usize,
        col: usize,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<ExpressionNode<'a>>,
        line: usize,
        col: usize,
    },
    Identifier(IdentifierNode<'a>),
    ArrayAccess {
        array: Box<ExpressionNode<'a>>,
        index: Box<ExpressionNode<'a>>,
        line: usize,
        col: usize,
    },
    MemberAccess {
        target: Box<ExpressionNode<'a>>,
        member: IdentifierNode<'a>,
        line: usize,
        col: usize,
    },
    FunctionCall {
        callee: Box<ExpressionNode<'a>>,
        args: Vec<ExpressionNode<'a>>,
        line: usize,
        col: usize,
    },
    LengthAccess {
        target: Box<ExpressionNode<'a>>,
        line: usize,
        col: usize,
    },
    Interpolation(StringInterpolationNode<'a>),
    Await {
        expr: Box<ExpressionNode<'a>>,
        line: usize,
        col: usize,
    },
    TypeName(TypeNameNode<'a>),
    RecordInit(RecordInitNode<'a>)
}

impl<'a> AstNode for ExpressionNode<'a> {
    fn line(&self) -> usize {
        match self {
            ExpressionNode::BinaryOp { line, .. } => *line,
            ExpressionNode::UnaryOp { line, .. } => *line,
            ExpressionNode::ArrayAccess { line, .. } => *line,
            ExpressionNode::Await { line, .. } => *line,
            ExpressionNode::Interpolation(node) => node.line,
            ExpressionNode::Literal(_) => 0,
            ExpressionNode::Identifier(id) => id.line,
            ExpressionNode::ArrayLiteral(_) => 0,
            ExpressionNode::TypeName(node) => node.line,
            ExpressionNode::MemberAccess { line, .. } => *line,
            ExpressionNode::FunctionCall { line, .. } => *line,
            ExpressionNode::LengthAccess { line, .. } => *line,
            ExpressionNode::RecordInit(node) => node.line,
        }
    }
    fn col(&self) -> usize {
        match self {
            ExpressionNode::BinaryOp { col, .. } => *col,
            ExpressionNode::UnaryOp { col, .. } => *col,
            ExpressionNode::ArrayAccess { col, .. } => *col,
            ExpressionNode::Await { col, .. } => *col,
            ExpressionNode::Interpolation(node) => node.col,
            ExpressionNode::Literal(_) => 0,
            ExpressionNode::Identifier(id) => id.col,
            ExpressionNode::ArrayLiteral(_) => 0,
            ExpressionNode::TypeName(node) => node.col,
            ExpressionNode::MemberAccess { col, .. } => *col,
            ExpressionNode::FunctionCall { col, .. } => *col,
            ExpressionNode::LengthAccess { col, .. } => *col,
            ExpressionNode::RecordInit(node) => node.col,
        }
    }
}

/// Statement AST (within functions)
#[derive(Debug, Clone)]
pub enum StatementNode<'a> {
    Expression(ExpressionNode<'a>),
    Let {
        name: IdentifierNode<'a>,
        type_name: TypeNameNode<'a>,
        expr: ExpressionNode<'a>,
        line: usize,
        col: usize,
    },
    Ask {
        prompt: StringInterpolationNode<'a>,
        target: Option<(IdentifierNode<'a>, TypeNameNode<'a>)>,
        line: usize,
        col: usize,
    },
    Say {
        expr: ExpressionNode<'a>,
        line: usize,
        col: usize,
    },
    Return {
        expr: Option<ExpressionNode<'a>>,
        line: usize,
        col: usize,
    },
    Bark {
        expr: ExpressionNode<'a>,
        line: usize,
        col: usize,
    },
    If(IfNode<'a>),
    Loop(LoopNode<'a>),
    Break {
        line: usize,
        col: usize,
    },
    Continue {
        line: usize,
        col: usize,
    },
    Import(ImportNode<'a>),
    ErrorHandling(ErrorHandlingNode<'a>),
    Assign {
        target: IdentifierNode<'a>,
        expr: ExpressionNode<'a>,
        line: usize,
        col: usize,
    },
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfNode<'a> {
    pub cond: ExpressionNode<'a>,
    pub then_block: Vec<StatementNode<'a>>,
    pub else_block: Option<Vec<StatementNode<'a>>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for IfNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Loop variants
#[derive(Debug, Clone)]
pub enum LoopNode<'a> {
    Infinite {
        body: Vec<StatementNode<'a>>,
        line: usize,
        col: usize,
    },
    While {
        cond: ExpressionNode<'a>,
        body: Vec<StatementNode<'a>>,
        line: usize,
        col: usize,
    },
    Range {
        var: IdentifierNode<'a>,
        start: ExpressionNode<'a>,
        end: ExpressionNode<'a>,
        body: Vec<StatementNode<'a>>,
        line: usize,
        col: usize,
    },
    Iterable {
        var: IdentifierNode<'a>,
        iterable: ExpressionNode<'a>,
        body: Vec<StatementNode<'a>>,
        line: usize,
        col: usize,
    },
}

impl<'a> AstNode for LoopNode<'a> {
    fn line(&self) -> usize {
        match self {
            LoopNode::Infinite { line, .. }
            | LoopNode::While { line, .. }
            | LoopNode::Range { line, .. }
            | LoopNode::Iterable { line, .. } => *line,
        }
    }
    fn col(&self) -> usize {
        match self {
            LoopNode::Infinite { col, .. }
            | LoopNode::While { col, .. }
            | LoopNode::Range { col, .. }
            | LoopNode::Iterable { col, .. } => *col,
        }
    }
}

/// Exception handling
#[derive(Debug, Clone)]
pub struct ErrorHandlingNode<'a> {
    pub sniff_body: Vec<StatementNode<'a>>,
    pub snatch_clauses: Vec<(IdentifierNode<'a>, Vec<StatementNode<'a>>)>,
    pub lastly_body: Option<Vec<StatementNode<'a>>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for ErrorHandlingNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Function definition
#[derive(Debug, Clone)]
pub struct FunctionDefinitionNode<'a> {
    pub is_async: bool,
    pub name: IdentifierNode<'a>,
    pub params: Vec<(IdentifierNode<'a>, TypeNameNode<'a>)>,
    pub return_type: TypeNameNode<'a>,
    pub body: Vec<StatementNode<'a>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for FunctionDefinitionNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Record definition
#[derive(Debug, Clone)]
pub struct RecordDefinitionNode<'a> {
    pub name: IdentifierNode<'a>,
    pub implements: Vec<IdentifierNode<'a>>,
    pub fields: Vec<(IdentifierNode<'a>, TypeNameNode<'a>)>,
    pub methods: Vec<FunctionDefinitionNode<'a>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for RecordDefinitionNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Protocol (interface) definition
#[derive(Debug, Clone)]
pub struct ProtocolDefinitionNode<'a> {
    pub name: IdentifierNode<'a>,
    pub methods: Vec<FunctionSignatureNode<'a>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for ProtocolDefinitionNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Function signature for protocols
#[derive(Debug, Clone)]
pub struct FunctionSignatureNode<'a> {
    pub is_async: bool,
    pub name: IdentifierNode<'a>,
    pub params: Vec<(IdentifierNode<'a>, TypeNameNode<'a>)>,
    pub return_type: TypeNameNode<'a>,
    pub line: usize,
    pub col: usize,
}

impl<'a> AstNode for FunctionSignatureNode<'a> {
    fn line(&self) -> usize {
        self.line
    }
    fn col(&self) -> usize {
        self.col
    }
}

/// Binary operators
#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    As,
}

/// Unary operators
#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}
