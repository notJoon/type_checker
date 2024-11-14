use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    ast::ASTNode,
    interpret::{merge_values, Merge},
};

/// abstract value
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AbstractValue {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Object(AbstractObject),
    Array(Vec<AbstractValue>),
    Union(Vec<AbstractValue>),
    Generic(String, Box<AbstractValue>), // String -> T, Box<AbstractValue> -> Concrete Type
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbstractObject {
    pub props: BTreeMap<String, AbstractValue>,
}

#[derive(Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub generics: Vec<(String, Option<String>)>,
    pub body: ASTNode,
}

#[derive(Clone)]
pub struct AbstractState {
    pub variables: HashMap<String, AbstractValue>,
    pub functions: HashMap<String, Function>,
}

////////////////////////////////////////////////////////////

impl Merge for AbstractValue {
    fn merge(&self, other: &Self) -> Self {
        use AbstractValue::*;

        // step1: identical => return self
        if self == other {
            return self.clone();
        }

        // step2: self is undefined => return other
        if matches!(self, Undefined) {
            return other.clone();
        }
        if matches!(other, Undefined) {
            return self.clone();
        }

        // Step 3: type-specific merging
        match (self, other) {
            // Array type
            (Array(a_elements), Array(b_elements)) => {
                let max_length = usize::max(a_elements.len(), b_elements.len());
                let merged_elements = (0..max_length)
                    .map(|i| {
                        let a_elem = a_elements.get(i).unwrap_or(&Undefined);
                        let b_elem = b_elements.get(i).unwrap_or(&Undefined);
                        a_elem.merge(b_elem)
                    })
                    .collect();
                Array(merged_elements)
            }
            // Object type
            (Object(a_obj), Object(b_obj)) => {
                let keys: HashSet<_> = a_obj.props.keys().chain(b_obj.props.keys()).collect();
                let merged_props = keys
                    .into_iter()
                    .map(|key| {
                        let a_val = a_obj.props.get(key).unwrap_or(&Undefined);
                        let b_val = b_obj.props.get(key).unwrap_or(&Undefined);
                        (key.clone(), a_val.merge(b_val))
                    })
                    .collect();
                Object(AbstractObject {
                    props: merged_props,
                })
            }
            // other cases => merge into Union
            _ => {
                let mut variants = HashSet::new();
                self.collect_variants(&mut variants);
                other.collect_variants(&mut variants);
                if variants.len() == 1 {
                    variants.into_iter().next().unwrap()
                } else {
                    Union(variants.into_iter().collect())
                }
            }
        }
    }
}

impl AbstractValue {
    fn collect_variants(&self, set: &mut HashSet<AbstractValue>) {
        match self {
            AbstractValue::Union(values) => {
                for v in values {
                    v.collect_variants(set);
                }
            }
            _ => {
                set.insert(self.clone());
            }
        }
    }
}

impl AbstractState {
    pub fn new() -> Self {
        AbstractState {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn assign(&mut self, name: &str, value: AbstractValue) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<&AbstractValue> {
        self.variables.get(name)
    }

    // e.g. for control flow
    pub fn merge(&mut self, other: &AbstractState) {
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
