use std::collections::HashMap;

use ast::ASTNode;
use interpret::{interpret, merge_values};

mod ast;
mod interpret;

/// abstract value
#[derive(Debug, Clone, PartialEq, Eq)]
enum AbstractValue {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Object(AbstractObject),
    Union(Vec<AbstractValue>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AbstractObject {
    props: HashMap<String, AbstractValue>,
}

#[derive(Clone)]
struct Function {
    params: Vec<String>,
    body: ASTNode,
}

#[derive(Clone)]
struct AbstractState {
    variables: HashMap<String, AbstractValue>,
    functions: HashMap<String, Function>,
}

impl AbstractState {
    fn new() -> Self {
        AbstractState {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn assign(&mut self, name: &str, value: AbstractValue) {
        self.variables.insert(name.to_string(), value);
    }

    fn get(&self, name: &str) -> Option<&AbstractValue> {
        self.variables.get(name)
    }

    // e.g. for control flow
    fn merge(&mut self, other: &AbstractState) {
        for (key, value) in &other.variables {
            if let Some(existing_value) = self.variables.get(key) {
                let merged_value = merge_values(existing_value, value);
                self.variables.insert(key.clone(), merged_value);
            } else {
                self.variables.insert(key.clone(), value.clone());
            }
        }
        for (key, function) in &other.functions {
            self.functions.insert(key.clone(), function.clone());
        }
    }
}

fn main() {
    let mut state = AbstractState::new();

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

    let program = ASTNode::Block {
        statements: vec![
            function_add,
            assign_x,
            assign_y,
            assign_z,
            if_statement,
            while_loop,
        ],
    };

    interpret(&program, &mut state);

    println!("Final state: {:?}", state.variables);
}
