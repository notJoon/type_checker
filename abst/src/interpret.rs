use std::collections::HashSet;

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
        ASTNode::ArrayLiteral(elements) => {
            let mut avv = Vec::new();
            for elem in elements {
                let value = interpret(elem, state);
                avv.push(value);
            }
            AbstractValue::Array(avv)
        },
        ASTNode::ArrayIndex { array, index } => {
            let array_value = interpret(array, state);
            let index_value = interpret(index, state);

            if !matches!(index_value, AbstractValue::Number) {
                return AbstractValue::Undefined;
            }

            let element_type = match array_value {
                AbstractValue::Array(elements) => {
                    // merge all element types in the array
                    let mut element_type = AbstractValue::Undefined;
                    for element in elements {
                        element_type = merge_values(&element_type, &element);
                    }
                    element_type
                }
                AbstractValue::Union(variants) => {
                    let mut element_type = AbstractValue::Undefined;
                    for variant in variants {
                        if let AbstractValue::Array(elements) = variant {
                            for element in elements {
                                element_type = merge_values(&element_type, &element);
                            }
                        } else {
                            // infer as undefined if not an array
                            element_type =
                                merge_values(&element_type, &AbstractValue::Undefined);
                        }
                    }
                    element_type
                }
                _ => AbstractValue::Undefined,
            };
            element_type
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
        return a.clone();
    }
    if matches!(a, AbstractValue::Undefined) {
        return b.clone();
    }
    if matches!(b, AbstractValue::Undefined) {
        return a.clone();
    }

    match (a, b) {
        // if both two values are Array, merge their each element
        (AbstractValue::Array(a_elements), AbstractValue::Array(b_elements)) => {
            let mut merged_elements = Vec::new();
            let max_length = usize::max(a_elements.len(), b_elements.len());
            for i in 0..max_length {
                // take i-th element from each array, if not exist, take Undefined
                let a_elem = a_elements.get(i).unwrap_or(&AbstractValue::Undefined);
                let b_elem = b_elements.get(i).unwrap_or(&AbstractValue::Undefined);
                let merged_elem = merge_values(a_elem, b_elem);
                merged_elements.push(merged_elem);
            }
            AbstractValue::Array(merged_elements)
        }
        // if one of them is Array, merge their each element
        (AbstractValue::Array(_), _) | (_, AbstractValue::Array(_)) => {
            merge_variants(vec![a.clone(), b.clone()])
        }
        // if both are same type, return the type
        _ => {
            merge_variants(vec![a.clone(), b.clone()])
        }
    }
}

// merge all variants into one
fn merge_variants(values: Vec<AbstractValue>) -> AbstractValue {
    let mut variants = HashSet::new();

    // recursively collect all variants to flaten the union
    fn collect_variants(value: AbstractValue, set: &mut HashSet<AbstractValue>) {
        match value {
            // we collect inner variants when we meet union
            AbstractValue::Union(values) => {
                for v in values {
                    collect_variants(v, set);
                }
            }
            _ => {
                set.insert(value);
            }
        }
    }

    for value in values {
        collect_variants(value, &mut variants);
    }

    if variants.len() == 1 {
        variants.into_iter().next().unwrap()
    } else {
        AbstractValue::Union(variants.into_iter().collect())
    }
}
