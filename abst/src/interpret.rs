use crate::{ast::ASTNode, types::Function, AbstractState, AbstractValue};

// This module performs abstract interpretation of an AST (Abstract Syntax Tree).
//
// The core functionality revolves around interpreting nodes using abstract values,
// which represent sets of possible runtime values.
//
// A key aspect of this interpretation is the merging of abstract values,
// especially when dealing with control flow constructs like if-statements and loops.
// The `merge` operation combines two abstract values into one, representing the union
// of their possible values.
//
// The `merge` operation is designed using algebraic properties to ensure consistency
// and correctness:
//
// - **Associativity**: `(a.merge(b)).merge(c) == a.merge(b.merge(c))`
//   - The grouping of merge operations does not affect the final result.
// - **Commutativity**: `a.merge(b) == b.merge(a)`
//   - The order of operands does not affect the merge result.
// - **Idempotence**: `a.merge(a) == a`
//   - Merging a value with itself yields the same value.
// - **Identity Element**: `Undefined` acts as the identity element.
//   - `a.merge(Undefined) == a` and `Undefined.merge(a) == a`
//
// By leveraging these algebraic properties, we ensure that the merging process
// is both predictable and (*ideally*) mathematically sound, which is crucial for accurate
// abstract interpretation.
//
// The `Merge` trait defines the `merge` method, and it is implemented for
// `AbstractValue`, allowing us to perform merges seamlessly across different
// abstract value types.

pub trait Merge {
    fn merge(&self, other: &Self) -> Self;
}

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
            then_value.merge(&else_value)
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
            let avv = elements.iter().map(|elem| interpret(elem, state)).collect();
            AbstractValue::Array(avv)
        }
        ASTNode::ArrayIndex { array, index } => {
            let array_value = interpret(array, state);
            let index_value = interpret(index, state);

            if !matches!(index_value, AbstractValue::Number) {
                return AbstractValue::Undefined;
            }

            let element_type = match array_value {
                AbstractValue::Array(elements) => {
                    // merge all elements
                    elements
                        .iter()
                        .fold(AbstractValue::Undefined, |acc, elem| acc.merge(elem))
                }
                AbstractValue::Union(variants) => {
                    variants
                        .iter()
                        .fold(AbstractValue::Undefined, |acc, variant| {
                            if let AbstractValue::Array(elements) = variant {
                                let elem_type = elements
                                    .iter()
                                    .fold(AbstractValue::Undefined, |e_acc, e| e_acc.merge(e));
                                acc.merge(&elem_type)
                            } else {
                                acc.merge(&AbstractValue::Undefined)
                            }
                        })
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
    a.merge(b)
}
