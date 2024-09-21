use ast::ASTNode;
use interpret::interpret;
use types::{AbstractState, AbstractValue};

mod ast;
mod interpret;
mod types;
fn main() {
    let mut state = AbstractState::new();

    // writing a parser to generate AST is unnecessary,

    // function add(a, b) { return a + b; }
    let function_add = ASTNode::FunctionDeclaration {
        name: "add".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        body: Box::new(ASTNode::BinaryOp {
            op: "+".to_string(),
            left: Box::new(ASTNode::Variable("a".to_string())),
            right: Box::new(ASTNode::Variable("b".to_string())),
        }),
    };

    // x = 10;
    let assign_x = ASTNode::Assignment {
        target: "x".to_string(),
        value: Box::new(ASTNode::Literal(AbstractValue::Number)),
    };

    // y = 20;
    let assign_y = ASTNode::Assignment {
        target: "y".to_string(),
        value: Box::new(ASTNode::Literal(AbstractValue::Number)),
    };

    // z = add(x, y);
    let assign_z = ASTNode::Assignment {
        target: "z".to_string(),
        value: Box::new(ASTNode::FunctionCall {
            function: Box::new(ASTNode::Variable("add".to_string())),
            arguments: vec![
                ASTNode::Variable("x".to_string()),
                ASTNode::Variable("y".to_string()),
            ],
        }),
    };

    // if (x == y) { w = "equal"; } else { w = 0; }
    let if_statement = ASTNode::IfStatement {
        condition: Box::new(ASTNode::BinaryOp {
            op: "==".to_string(),
            left: Box::new(ASTNode::Variable("x".to_string())),
            right: Box::new(ASTNode::Variable("y".to_string())),
        }),
        then_branch: Box::new(ASTNode::Assignment {
            target: "w".to_string(),
            value: Box::new(ASTNode::Literal(AbstractValue::String)),
        }),
        else_branch: Some(Box::new(ASTNode::Assignment {
            target: "w".to_string(),
            value: Box::new(ASTNode::Literal(AbstractValue::Number)),
        })),
    };

    // while (i < 10) { i = i + 1; }
    let while_loop = ASTNode::WhileLoop {
        condition: Box::new(ASTNode::BinaryOp {
            op: "<".to_string(),
            left: Box::new(ASTNode::Variable("i".to_string())),
            right: Box::new(ASTNode::Literal(AbstractValue::Number)),
        }),
        body: Box::new(ASTNode::Assignment {
            target: "i".to_string(),
            value: Box::new(ASTNode::BinaryOp {
                op: "+".to_string(),
                left: Box::new(ASTNode::Variable("i".to_string())),
                right: Box::new(ASTNode::Literal(AbstractValue::Number)),
            }),
        }),
    };

    // arr = [1, "two", true];
    let assign_arr = ASTNode::Assignment {
        target: "arr".to_string(),
        value: Box::new(ASTNode::ArrayLiteral(vec![
            ASTNode::Literal(AbstractValue::Number),
            ASTNode::Literal(AbstractValue::String),
            ASTNode::Literal(AbstractValue::Boolean),
        ])),
    };

    // elem = arr[0];
    let assign_elem = ASTNode::Assignment {
        target: "elem".to_string(),
        value: Box::new(ASTNode::ArrayIndex {
            array: Box::new(ASTNode::Variable("arr".to_string())),
            index: Box::new(ASTNode::Literal(AbstractValue::Number)), // 인덱스는 숫자로 처리
        }),
    };

    let program = ASTNode::Block {
        statements: vec![
            function_add,
            assign_x,
            assign_y,
            assign_z,
            if_statement,
            while_loop,
            assign_arr,
            assign_elem,
        ],
    };

    interpret(&program, &mut state);

    println!("Final state: {:?}", state.variables);
}
