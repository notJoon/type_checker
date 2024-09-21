use crate::{ast::ASTNode, AbstractState, AbstractValue, Function};

// Abstract interpretation
pub fn interpret(node: &ASTNode, state: &mut AbstractState) -> AbstractValue {
    match node {
        ASTNode::Literal(value) => value.clone(),
        ASTNode::Variable(name) => state.get(name).cloned().unwrap_or(AbstractValue::Undefined),
        ASTNode::Assignment { target, value } => {
            let abstract_value = interpret(value, state);
            state.assign(target, abstract_value.clone());
            abstract_value
        }
        ASTNode::BinaryOp { op, left, right } => {
            let left_value = interpret(left, state);
            let right_value = interpret(right, state);
            // perform abstract operation based on operator
            match op.as_str() {
                "+" => abstract_add(&left_value, &right_value),
                "-" => abstract_subtract(&left_value, &right_value),
                "*" => abstract_multiply(&left_value, &right_value),
                "/" => abstract_divide(&left_value, &right_value),
                "==" => abstract_equal(&left_value, &right_value),
                _ => AbstractValue::Undefined,
            }
        }
        ASTNode::IfStatement {
            condition,
            then_branch,
            else_branch,
        } => {
            let _condition_value = interpret(condition, state);
            // consider both paths in the if statement
            let mut then_state = state.clone();
            let mut else_state = state.clone();
            let then_value = interpret(then_branch, &mut then_state);
            let else_value = if let Some(else_branch) = else_branch {
                interpret(else_branch, &mut else_state)
            } else {
                AbstractValue::Undefined
            };
            // merge states
            state.merge(&then_state);
            state.merge(&else_state);
            merge_values(&then_value, &else_value)
        }
        ASTNode::WhileLoop { condition: _, body } => {
            // assume loop runs 0 or more times
            let mut loop_state = state.clone();
            interpret(body, &mut loop_state);
            state.merge(&loop_state);
            AbstractValue::Undefined
        }
        ASTNode::Block { statements } => {
            let mut result = AbstractValue::Undefined;
            for stmt in statements {
                result = interpret(stmt, state);
            }
            result
        }
        ASTNode::FunctionDeclaration { name, params, body } => {
            let function = Function {
                params: params.clone(),
                body: *body.clone(),
            };
            state.functions.insert(name.clone(), function);
            AbstractValue::Undefined
        }
        ASTNode::FunctionCall {
            function,
            arguments,
        } => {
            // we assume function is a variable name for simplicity
            if let ASTNode::Variable(func_name) = &**function {
                if let Some(func) = state.functions.get(func_name).cloned() {
                    // immutable borrow ends here
                    // create new state
                    let mut func_state = AbstractState::new();
                    // assign arguments to parameters
                    for (param, arg_node) in func.params.iter().zip(arguments.iter()) {
                        let arg_value = interpret(arg_node, state);
                        func_state.assign(param, arg_value);
                    }
                    // interpret function body
                    interpret(&func.body, &mut func_state)
                } else {
                    AbstractValue::Undefined
                }
            } else {
                AbstractValue::Undefined
            }
        }
    }
}

fn abstract_add(left: &AbstractValue, right: &AbstractValue) -> AbstractValue {
    match (left, right) {
        (AbstractValue::Number, AbstractValue::Number) => AbstractValue::Number,
        (AbstractValue::String, _) | (_, AbstractValue::String) => AbstractValue::String,
        _ => AbstractValue::Union(vec![AbstractValue::Number, AbstractValue::String]),
    }
}

fn abstract_subtract(left: &AbstractValue, right: &AbstractValue) -> AbstractValue {
    if matches!(left, AbstractValue::Number) && matches!(right, AbstractValue::Number) {
        AbstractValue::Number
    } else {
        AbstractValue::Undefined
    }
}

fn abstract_multiply(left: &AbstractValue, right: &AbstractValue) -> AbstractValue {
    if matches!(left, AbstractValue::Number) && matches!(right, AbstractValue::Number) {
        AbstractValue::Number
    } else {
        AbstractValue::Undefined
    }
}

fn abstract_divide(left: &AbstractValue, right: &AbstractValue) -> AbstractValue {
    if matches!(left, AbstractValue::Number) && matches!(right, AbstractValue::Number) {
        AbstractValue::Number
    } else {
        AbstractValue::Undefined
    }
}

fn abstract_equal(_left: &AbstractValue, _right: &AbstractValue) -> AbstractValue {
    AbstractValue::Boolean
}

pub fn merge_values(a: &AbstractValue, b: &AbstractValue) -> AbstractValue {
    if a == b {
        a.clone()
    } else {
        match (a, b) {
            (AbstractValue::Union(av), AbstractValue::Union(bv)) => {
                let mut union = av.clone();
                for v in bv {
                    if !union.contains(v) {
                        union.push(v.clone());
                    }
                }
                AbstractValue::Union(union)
            }
            (AbstractValue::Union(av), _) => {
                let mut union = av.clone();
                if !union.contains(b) {
                    union.push(b.clone());
                }
                AbstractValue::Union(union)
            }
            (_, AbstractValue::Union(bv)) => {
                let mut union = bv.clone();
                if !union.contains(a) {
                    union.push(a.clone());
                }
                AbstractValue::Union(union)
            }
            _ => AbstractValue::Union(vec![a.clone(), b.clone()]),
        }
    }
}
