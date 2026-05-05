#![allow(dead_code)]
use crate::{
    ir::{
        builder::IRBuilder,
        types::{Branch, Env, Operation, PendingEdge, TIRFunction, Terminator, VirtualRegister},
    },
    parser::{BinOp, ElseClause, Expression, Function, Statement, Term},
};

fn lower_function(function: Function) -> TIRFunction {
    let mut builder = IRBuilder::new();
    let mut env = Env::new();

    let params = function
        .args
        .iter()
        .map(|arg| {
            let value = builder.value();
            env.insert(arg.name.name.clone(), value);
            value
        })
        .collect::<Vec<_>>();
    builder.current_mut().params = params.clone();

    lower_scope(function.body, env, &mut builder);

    TIRFunction {
        name: function.name.name,
        params,
        blocks: builder.to_blocks(),
    }
}

fn lower_scope(scope: Vec<Statement>, mut env: Env, builder: &mut IRBuilder) -> Option<Env> {
    for statement in scope {
        env = lower_statement(statement, env.clone(), builder)?;
    }

    Some(env)
}

fn lower_statement(statement: Statement, mut env: Env, builder: &mut IRBuilder) -> Option<Env> {
    match statement {
        Statement::Exit(expr) => {
            let v = lower_expr(expr, &env, builder);
            builder.terminate(Terminator::Exit(v));
            None
        }
        Statement::Let { ident, expr } | Statement::Assignment { ident, expr } => {
            let v = lower_expr(expr, &env, builder);
            env.insert(ident.name, v);
            Some(env)
        }
        Statement::If { cond, then, els } => Some(lower_if(cond, then, els, env, builder)),
        Statement::While { cond, body } => Some(lower_while(cond, body, env, builder)),
        Statement::FunctionCall { name, args } => {
            let arg_vals = args
                .into_iter()
                .map(|a| lower_expr(a, &env, builder))
                .collect();

            let _result = builder.emit(Operation::Call(name.name.clone(), arg_vals));
            Some(env)
        }
        Statement::Return(expr) => {
            let v = lower_expr(expr, &env, builder);
            builder.terminate(Terminator::Return(v));
            None
        }
    }
}

fn lower_if(
    cond: Expression,
    then_body: Vec<Statement>,
    else_clause: Option<ElseClause>,
    env: Env,
    builder: &mut IRBuilder,
) -> Env {
    let branches = flatten_if_chain(cond, then_body, else_clause);
    let vars: Vec<String> = env.keys().cloned().collect();

    let mut pending_edges = vec![];
    let mut next_block = builder.current();

    for (cond_opt, body) in branches.into_iter() {
        if let Some(cond_expr) = cond_opt {
            let then_block = builder.init_block();
            let else_block = builder.init_block();

            builder.switch_to(next_block);
            let cond_v = lower_expr(cond_expr, &env, builder);
            builder.terminate(Terminator::ConditionalBranch {
                cond: cond_v,
                then_target: then_block,
                then_params: vec![],
                else_target: else_block,
                else_params: vec![],
            });

            builder.switch_to(then_block);
            let then_env = lower_scope(body, env.clone(), builder);
            if let Some(env_out) = then_env {
                let vals = vars.iter().map(|v| env_out[v]).collect::<Vec<_>>();
                pending_edges.push(PendingEdge {
                    from: then_block,
                    params: vals,
                });
            }

            next_block = else_block;
        } else {
            // final else (no condition)
            builder.switch_to(next_block);
            let else_env = lower_scope(body, env.clone(), builder);
            if let Some(env_out) = else_env {
                let vals = vars.iter().map(|v| env_out[v]).collect::<Vec<_>>();
                pending_edges.push(PendingEdge {
                    from: next_block,
                    params: vals,
                });
            }
        }
    }

    // If no branches reach merge
    if pending_edges.is_empty() {
        return env;
    }

    let merge_block = builder.init_block();
    for edge in pending_edges {
        builder.switch_to(edge.from);
        builder.terminate(Terminator::Branch {
            target: merge_block,
            params: edge.params,
        });
    }
    builder.switch_to(merge_block);

    let params = (0..vars.len()).map(|_| builder.value()).collect::<Vec<_>>();
    builder.current_mut().params = params.clone();
    let mut new_env = Env::new();
    for (i, var) in vars.iter().enumerate() {
        new_env.insert(var.clone(), params[i]);
    }

    new_env
}

fn flatten_if_chain(
    cond: Expression,
    then: Vec<Statement>,
    els: Option<ElseClause>,
) -> Vec<Branch> {
    let mut branches = vec![(Some(cond), then)];
    let mut current = els;
    while let Some(clause) = current {
        branches.push((clause.cond, clause.body.clone()));
        current = *clause.els;
    }

    if branches.last().is_some_and(|x| x.0.is_some()) {
        branches.push((None, vec![]));
    }

    branches
}

fn lower_while(cond: Expression, body: Vec<Statement>, env: Env, builder: &mut IRBuilder) -> Env {
    let vars: Vec<String> = env.keys().cloned().collect();

    let cond_block = builder.init_block();
    let body_block = builder.init_block();
    let done_block = builder.init_block();

    let init_vars = vars.iter().map(|v| env[v]).collect();
    builder.terminate(Terminator::Branch {
        target: cond_block,
        params: init_vars,
    });

    let cond_params = (0..vars.len()).map(|_| builder.value()).collect::<Vec<_>>();
    builder.switch_to(cond_block);
    builder.current_mut().params = cond_params.clone();
    let mut cond_env = Env::new();
    for (i, var) in vars.iter().enumerate() {
        cond_env.insert(var.clone(), cond_params[i]);
    }
    let cond_v = lower_expr(cond, &cond_env, builder);
    builder.terminate(Terminator::ConditionalBranch {
        cond: cond_v,
        then_target: body_block,
        then_params: cond_params.clone(),
        else_target: done_block,
        else_params: cond_params.clone(),
    });

    builder.switch_to(body_block);
    let body_env = lower_scope(body, cond_env.clone(), builder).unwrap();
    let updated_vals = vars.iter().map(|v| body_env[v]).collect();
    builder.terminate(Terminator::Branch {
        target: cond_block,
        params: updated_vals,
    });

    let exit_params = (0..vars.len()).map(|_| builder.value()).collect::<Vec<_>>();
    builder.switch_to(done_block);
    builder.current_mut().params = exit_params.clone();
    let mut exit_env = Env::new();
    for (i, var) in vars.iter().enumerate() {
        exit_env.insert(var.clone(), exit_params[i]);
    }

    exit_env
}

fn lower_expr(expr: Expression, env: &Env, builder: &mut IRBuilder) -> VirtualRegister {
    match expr {
        Expression::BinaryExpr(lhs, rhs, op) => {
            let lhs = lower_expr(*lhs, env, builder);
            let rhs = lower_expr(*rhs, env, builder);

            let operation = match op {
                BinOp::Add => Operation::Add(lhs, rhs),
                BinOp::Sub => Operation::Sub(lhs, rhs),
                BinOp::Mul => Operation::Mul(lhs, rhs),
                BinOp::And => Operation::And(lhs, rhs),
                BinOp::Or => Operation::Or(lhs, rhs),
                BinOp::Lt => Operation::Lt(lhs, rhs),
                BinOp::Leq => Operation::Leq(lhs, rhs),
                BinOp::Gt => Operation::Gt(lhs, rhs),
                BinOp::Geq => Operation::Geq(lhs, rhs),
                BinOp::Eq => Operation::Eq(lhs, rhs),
                BinOp::Neq => Operation::Neq(lhs, rhs),
            };

            builder.emit(operation)
        }
        Expression::Term(term) => match term {
            Term::Identifier(name) => env.get(&name).expect("identifier must exist").to_owned(),
            Term::IntLit(n) => builder.emit(Operation::ConstInt(n)),
            Term::Bool(b) => builder.emit(Operation::ConstBool(b)),
        },
        Expression::FunctionCall { name, args } => {
            let args = args
                .into_iter()
                .map(|a| lower_expr(a, env, builder))
                .collect();

            builder.emit(Operation::Call(name.name, args))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{Identifier, Type};

    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let func = Function {
            name: ident("main"),
            args: vec![],
            ret_sig: Type::Int,
            body: vec![
                Statement::Let {
                    ident: ident("x"),
                    expr: bin(int(1), int(2), BinOp::Add),
                },
                Statement::Return(var("x")),
            ],
        };

        let tir = lower_function(func);

        assert_eq!(tir.blocks.len(), 1);
        assert!(count_instructions(&tir, |op| matches!(op, Operation::Add(_, _))) == 1);
        assert!(find_block_with_terminator(&tir, |t| matches!(
            t,
            Terminator::Return(_)
        )));
    }

    #[test]
    fn test_function_call_statement() {
        let func = Function {
            name: ident("main"),
            args: vec![],
            ret_sig: Type::Int,
            body: vec![
                Statement::FunctionCall {
                    name: ident("foo"),
                    args: vec![int(1)],
                },
                Statement::Return(int(0)),
            ],
        };

        let tir = lower_function(func);
        assert!(count_instructions(&tir, |op| matches!(op, Operation::Call(_, _))) == 1);
    }

    #[test]
    fn test_if_without_else() {
        let func = Function {
            name: ident("main"),
            args: vec![],
            ret_sig: Type::Int,
            body: vec![
                Statement::Let {
                    ident: ident("x"),
                    expr: int(1),
                },
                Statement::If {
                    cond: bin(var("x"), int(0), BinOp::Gt),
                    then: vec![Statement::Assignment {
                        ident: ident("x"),
                        expr: int(2),
                    }],
                    els: None,
                },
                Statement::Return(var("x")),
            ],
        };

        let tir = lower_function(func);

        // entry, if, identity-else merge
        assert_eq!(tir.blocks.len(), 4);
        assert!(find_block_with_terminator(&tir, |t| {
            matches!(t, Terminator::ConditionalBranch { .. })
        }));
    }

    #[test]
    fn test_if_else() {
        let func = Function {
            name: ident("main"),
            args: vec![],
            ret_sig: Type::Int,
            body: vec![
                Statement::Let {
                    ident: ident("x"),
                    expr: int(1),
                },
                Statement::If {
                    cond: bool(true),
                    then: vec![Statement::Assignment {
                        ident: ident("x"),
                        expr: int(2),
                    }],
                    els: Some(ElseClause {
                        cond: None,
                        body: vec![Statement::Assignment {
                            ident: ident("x"),
                            expr: int(3),
                        }],
                        els: Box::new(None),
                    }),
                },
                Statement::Return(var("x")),
            ],
        };

        let tir = lower_function(func);
        // entry, if, else, merge
        assert_eq!(tir.blocks.len(), 4);
    }

    #[test]
    fn test_else_if_chain() {
        let func = Function {
            name: ident("main"),
            args: vec![],
            ret_sig: Type::Int,
            body: vec![Statement::If {
                cond: bool(false),
                then: vec![Statement::Return(int(1))],
                els: Some(ElseClause {
                    cond: Some(bool(false)),
                    body: vec![Statement::Return(int(2))],
                    els: Box::new(Some(ElseClause {
                        cond: None,
                        body: vec![Statement::Return(int(3))],
                        els: Box::new(None),
                    })),
                }),
            }],
        };

        let tir = lower_function(func);
        assert_eq!(count_blocks(&tir), 5);
    }

    #[test]
    fn test_while_loop() {
        let func = Function {
            name: ident("main"),
            args: vec![],
            ret_sig: Type::Int,
            body: vec![
                Statement::Let {
                    ident: ident("x"),
                    expr: int(0),
                },
                Statement::While {
                    cond: bin(var("x"), int(10), BinOp::Lt),
                    body: vec![Statement::Assignment {
                        ident: ident("x"),
                        expr: bin(var("x"), int(1), BinOp::Add),
                    }],
                },
                Statement::Return(var("x")),
            ],
        };

        let tir = lower_function(func);
        // entry, cond, body, done
        assert_eq!(tir.blocks.len(), 4);
        assert!(find_block_with_terminator(&tir, |t| {
            matches!(t, Terminator::Branch { .. })
        }));
    }

    fn count_blocks(func: &TIRFunction) -> usize {
        func.blocks.len()
    }

    fn find_block_with_terminator<F>(func: &TIRFunction, f: F) -> bool
    where
        F: Fn(&Terminator) -> bool,
    {
        func.blocks.iter().any(|b| f(&b.terminator))
    }

    fn count_instructions<F>(func: &TIRFunction, f: F) -> usize
    where
        F: Fn(&Operation) -> bool,
    {
        func.blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .filter(|instr| f(&instr.op))
            .count()
    }

    fn ident(name: &str) -> Identifier {
        Identifier { name: name.into() }
    }

    fn int(n: usize) -> Expression {
        Expression::Term(Term::IntLit(n))
    }

    fn bool(b: bool) -> Expression {
        Expression::Term(Term::Bool(b))
    }

    fn var(name: &str) -> Expression {
        Expression::Term(Term::Identifier(name.into()))
    }

    fn bin(lhs: Expression, rhs: Expression, op: BinOp) -> Expression {
        Expression::BinaryExpr(Box::new(lhs), Box::new(rhs), op)
    }
}
