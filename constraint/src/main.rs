use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Type {
    Int,
    Bool,
    Var(TypeVar),
    Func(Box<Type>, Box<Type>), // params, return
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TypeVar(usize);

// AST node
#[derive(Debug)]
enum Expr {
    IntLiteral(i32),
    BoolLiteral(bool),
    Variable(String),
    Lambda {
        param: String,
        body: Box<Expr>,
    },
    Application {
        func: Box<Expr>,
        arg: Box<Expr>,
    },
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

// context for type inference
struct TypeContext {
    next_var_id: usize,
    substitutions: HashMap<TypeVar, Type>,
    env: HashMap<String, Type>,
}

impl TypeContext {
    fn new() -> Self {
        TypeContext {
            next_var_id: 0,
            substitutions: HashMap::new(),
            env: HashMap::new(),
        }
    }

    // create a new type variable
    fn new_type_var(&mut self) -> Type {
        let var = TypeVar(self.next_var_id);
        self.next_var_id += 1;
        Type::Var(var)
    }

    // find type variable's real type
    fn lookup_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Var(tv) => {
                if let Some(t_sub) = self.substitutions.get(tv) {
                    let t_sub_clone = t_sub.clone();
                    let t_sub_final = self.lookup_type(&t_sub_clone);
                    self.substitutions.insert(tv.clone(), t_sub_final.clone());
                    t_sub_final
                } else {
                    t.clone()
                }
            }
            _ => t.clone(),
        }
    }

    // unifying two types and resolve constraint
    fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), String> {
        let a = self.lookup_type(t1);
        let b = self.lookup_type(t2);

        match (&a, &b) {
            (&Type::Int, &Type::Int) | (&Type::Bool, &Type::Bool) => Ok(()),
            (&Type::Var(ref tv), t) | (t, &Type::Var(ref tv)) => {
                let t = t.clone();
                if t == Type::Var(tv.clone()) {
                    Ok(())
                } else if occurs_check(tv, &t, self) {
                    Err(format!("Occurs check failed for {:?} in {:?}", tv, t))
                } else {
                    self.substitutions.insert(tv.clone(), t);
                    Ok(())
                }
            }
            (&Type::Func(ref a1, ref a2), &Type::Func(ref b1, ref b2)) => {
                self.unify(&*a1, &*b1)?;
                self.unify(&*a2, &*b2)
            }
            _ => Err(format!("Type mismatch: {:?} vs {:?}", a, b)),
        }
    }
}

fn occurs_check(var: &TypeVar, ty: &Type, ctx: &mut TypeContext) -> bool {
    match ty {
        Type::Var(tv) => {
            let t = ctx.lookup_type(ty);
            match t {
                Type::Var(tv2) => tv == &tv2,
                _ => occurs_check(var, &t, ctx),
            }
        }
        Type::Func(t1, t2) => occurs_check(var, &t1, ctx) || occurs_check(var, &t2, ctx),
        _ => false,
    }
}

fn infer(expr: &Expr, ctx: &mut TypeContext) -> Result<Type, String> {
    match expr {
        Expr::IntLiteral(_) => Ok(Type::Int),
        Expr::BoolLiteral(_) => Ok(Type::Bool),
        Expr::Variable(name) => {
            if let Some(ty) = ctx.env.get(name) {
                Ok(ty.clone())
            } else {
                Err(format!("Unbound variable: {}", name))
            }
        }
        Expr::Lambda { param, body } => {
            let param_type = ctx.new_type_var();
            ctx.env.insert(param.clone(), param_type.clone());
            let body_type = infer(body, ctx)?;
            ctx.env.remove(param);
            Ok(Type::Func(Box::new(param_type), Box::new(body_type)))
        }
        Expr::Application { func, arg } => {
            let func_type = infer(func, ctx)?;
            let arg_type = infer(arg, ctx)?;
            let result_type = ctx.new_type_var();
            ctx.unify(
                &func_type,
                &Type::Func(Box::new(arg_type), Box::new(result_type.clone())),
            )?;
            Ok(result_type)
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_type = infer(cond, ctx)?;
            ctx.unify(&cond_type, &Type::Bool)?;
            let then_type = infer(then_branch, ctx)?;
            let else_type = infer(else_branch, ctx)?;
            ctx.unify(&then_type, &else_type)?;
            Ok(then_type)
        }
        Expr::Let { name, value, body } => {
            let value_type = infer(value, ctx)?;
            ctx.env.insert(name.clone(), value_type);
            let body_type = infer(body, ctx)?;
            ctx.env.remove(name);
            Ok(body_type)
        } // 다른 표현식에 대한 처리...
    }
}

// apply substitutions to get the actual type
fn apply_substitutions(ty: &Type, ctx: &mut TypeContext) -> Type {
    match ty {
        Type::Var(_) => ctx.lookup_type(ty),
        Type::Func(t1, t2) => Type::Func(
            Box::new(apply_substitutions(t1, ctx)),
            Box::new(apply_substitutions(t2, ctx)),
        ),
        _ => ty.clone(),
    }
}

// convert type to string for output
fn type_to_string(ty: &Type, ctx: &mut TypeContext) -> String {
    match ty {
        Type::Int => "Int".to_string(),
        Type::Bool => "Bool".to_string(),
        Type::Var(tv) => {
            let actual_type = ctx.lookup_type(&Type::Var(tv.clone()));
            if let Type::Var(_) = actual_type {
                format!("t{}", tv.0)
            } else {
                type_to_string(&actual_type, ctx)
            }
        }
        Type::Func(t1, t2) => format!(
            "({} -> {})",
            type_to_string(t1, ctx),
            type_to_string(t2, ctx)
        ),
    }
}

fn main() {
    let mut ctx = TypeContext::new();

    // assume '+' operator as a function and add to environment
    ctx.env.insert(
        "+".to_string(),
        Type::Func(
            Box::new(Type::Int),
            Box::new(Type::Func(Box::new(Type::Int), Box::new(Type::Int))),
        ),
    );

    // let add = λx.λy.x + y => add 1 2
    let expr = Expr::Let {
        name: "add".to_string(),
        value: Box::new(Expr::Lambda {
            param: "x".to_string(),
            body: Box::new(Expr::Lambda {
                param: "y".to_string(),
                body: Box::new(Expr::Application {
                    func: Box::new(Expr::Application {
                        func: Box::new(Expr::Variable("+".to_string())),
                        arg: Box::new(Expr::Variable("x".to_string())),
                    }),
                    arg: Box::new(Expr::Variable("y".to_string())),
                }),
            }),
        }),
        body: Box::new(Expr::Application {
            func: Box::new(Expr::Application {
                func: Box::new(Expr::Variable("add".to_string())),
                arg: Box::new(Expr::IntLiteral(1)),
            }),
            arg: Box::new(Expr::IntLiteral(2)),
        }),
    };

    match infer(&expr, &mut ctx) {
        Ok(ty) => {
            let final_type = apply_substitutions(&ty, &mut ctx);
            println!("Expression Type: {}", type_to_string(&final_type, &mut ctx));
        }
        Err(err) => {
            println!("Type inference error: {}", err);
        }
    }
}
