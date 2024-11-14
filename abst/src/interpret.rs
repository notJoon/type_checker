use std::collections::HashMap;

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
        ASTNode::FunctionDeclaration {
            name,
            params,
            generics,
            body,
        } => {
            // when we encounter a function declaration, we construct a `Function` struct.
            // the struct stores the function's params, optionally its generic types (along with any constrains), and its body.
            //
            // Then, insert this function into the current state, associating it with its identifier.
            // This allows us to later retrieve and call this function during interpretation process.
            //
            // Example:
            // ```
            // function add<T: Number>(a: T, b: T) {
            //     return a + b;
            // }
            // ```
            //
            // This function `add` takes two parameters `a` and `b` of generic type `T`
            // constrained to be a `Number`.
            let function = Function {
                params: params.clone(),
                generics: generics.clone(),
                body: *body.clone(),
            };
            // store the function in the state to allow it to be invoked later
            state.functions.insert(name.clone(), function);
            // return `Undefined` since defining a function
            // does not produce a value immediately.
            AbstractValue::Undefined
        }
        ASTNode::FunctionCall {
            function,
            arguments,
        } => {
            // When we encounter a function call, we assume that the `function` field contains
            // the variable name of the function.
            //
            // Example:
            // ```
            // result = add(5, 10);
            // ```
            //
            // Here, `add` is the variable name of the function.
            if let ASTNode::Variable(func_name) = &**function {
                // look up the function by its name in the current state
                if let Some(func) = state.functions.get(func_name).cloned() {
                    // create new abstract state for interpreting this function call.
                    // this represents the local state/context within the function body.
                    let mut func_state = AbstractState::new();

                    // bind the provided arguments to the function's parameters.
                    for (param, arg_node) in func.params.iter().zip(arguments.iter()) {
                        let arg_value = interpret(arg_node, state);
                        func_state.assign(param, arg_value);
                    }

                    // create a mapping of generic type parameters to concrete values provided during the call.
                    let mut generic_mapping = HashMap::new();
                    for (i, (generic, constraint)) in func.generics.iter().enumerate() {
                        // for each generic parameter, retrieve the corresponding argument if available.
                        if let Some(arg_node) = arguments.get(i) {
                            let arg_value = interpret(arg_node, state);

                            // check constraint
                            if let Some(constraint_type) = constraint {
                                if !satisfies_constraint(&arg_value, &constraint_type) {
                                    // if the argument does not satisfy the constraint, return undefined
                                    return AbstractValue::Undefined;
                                }
                            }

                            // mapping the generic to the argument's type when satisfied constraint
                            generic_mapping.insert(generic.clone(), Box::new(arg_value));
                        }
                    }

                    // re-assign parameters with their arguments within the new function state for evaluation
                    for (param, arg_node) in func.params.iter().zip(arguments.iter()) {
                        let arg_value = interpret(arg_node, state);
                        func_state.assign(param, arg_value);
                    }

                    // Interpret the function body using the newly created function state.
                    // During this step, any references to generics should be replaced with their concrete types.
                    // This ensures that the function body operates with the correct types.
                    let result = interpret(&func.body, &mut func_state);

                    // return the result of interpreting the function body.
                    //
                    // TODO: if needed connect the result with concrete generics
                    return result;
                }
                // not found in state
                return AbstractValue::Undefined;
            }
            AbstractValue::Undefined
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
                                return acc.merge(&elem_type);
                            }
                            acc.merge(&AbstractValue::Undefined)
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

// check if the value satisfies the constraint
fn satisfies_constraint(v: &AbstractValue, constraint: &str) -> bool {
    match constraint {
        "Number" => matches!(v, AbstractValue::Number),
        "String" => matches!(v, AbstractValue::String),
        "Boolean" => matches!(v, AbstractValue::Boolean),
        // TODO: add more constraints
        _ => false,
    }
}

#[cfg(test)]
mod interpreter_tests {
    use super::*;
    use crate::ast::ASTNode;
    use crate::types::{AbstractState, AbstractValue};

    #[test]
    fn test_generic_function_call() {
        let mut state = AbstractState::new();

        // function identity<T>(x: T) { return x; }
        let function_identity = ASTNode::FunctionDeclaration {
            name: "identity".to_string(),
            params: vec!["x".to_string()],
            generics: vec![("T".to_string(), None)],
            body: Box::new(ASTNode::Variable("x".to_string())),
        };

        interpret(&function_identity, &mut state);

        // y = identity(42);
        let call_identity_number = ASTNode::Assignment {
            target: "y".to_string(),
            value: Box::new(ASTNode::FunctionCall {
                function: Box::new(ASTNode::Variable("identity".to_string())),
                arguments: vec![ASTNode::Literal(AbstractValue::Number)],
            }),
        };

        interpret(&call_identity_number, &mut state);

        assert_eq!(
            state.get("y").cloned().unwrap(),
            AbstractValue::Number,
            "Expected y to be a Number"
        );

        // z = identity("hello");
        let call_identity_string = ASTNode::Assignment {
            target: "z".to_string(),
            value: Box::new(ASTNode::FunctionCall {
                function: Box::new(ASTNode::Variable("identity".to_string())),
                arguments: vec![ASTNode::Literal(AbstractValue::String)],
            }),
        };

        interpret(&call_identity_string, &mut state);

        assert_eq!(
            state.get("z").cloned().unwrap(),
            AbstractValue::String,
            "Expected z to be a String"
        );
    }

    #[test]
    fn test_bounded_generic_function_call() {
        let mut state = AbstractState::new();

        // function add<T: Number>(a: T, b: T) { return a + b; }
        let function_add = ASTNode::FunctionDeclaration {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            generics: vec![("T".to_string(), Some("Number".to_string()))], // generic with constraint
            body: Box::new(ASTNode::BinaryOp {
                op: "+".to_string(),
                left: Box::new(ASTNode::Variable("a".to_string())),
                right: Box::new(ASTNode::Variable("b".to_string())),
            }),
        };

        interpret(&function_add, &mut state);

        // result = add(5, 10);
        let call_add_correct = ASTNode::Assignment {
            target: "result".to_string(),
            value: Box::new(ASTNode::FunctionCall {
                function: Box::new(ASTNode::Variable("add".to_string())),
                arguments: vec![
                    ASTNode::Literal(AbstractValue::Number),
                    ASTNode::Literal(AbstractValue::Number),
                ],
            }),
        };

        interpret(&call_add_correct, &mut state);

        assert_eq!(
            state.get("result").cloned().unwrap(),
            AbstractValue::Number,
            "Expected result to be a Number"
        );

        // invalid_result = add("hello", 10);
        let call_add_invalid = ASTNode::Assignment {
            target: "invalid_result".to_string(),
            value: Box::new(ASTNode::FunctionCall {
                function: Box::new(ASTNode::Variable("add".to_string())),
                arguments: vec![
                    ASTNode::Literal(AbstractValue::String),
                    ASTNode::Literal(AbstractValue::Number),
                ],
            }),
        };

        interpret(&call_add_invalid, &mut state);

        assert_eq!(
            state.get("invalid_result").cloned().unwrap(),
            AbstractValue::Undefined,
            "Expected invalid_result to be Undefined due to type mismatch"
        );
    }
}
