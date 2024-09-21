use crate::AbstractValue;

#[derive(Clone)]
pub enum ASTNode {
    Literal(AbstractValue),
    Variable(String),
    Assignment {
        target: String,
        value: Box<ASTNode>,
    },
    BinaryOp {
        op: String,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    IfStatement {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    WhileLoop {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    Block {
        statements: Vec<ASTNode>,
    },
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Box<ASTNode>,
    },
    FunctionCall {
        function: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    ArrayLiteral(Vec<ASTNode>),
    ArrayIndex {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
    },
}
