#![allow(dead_code)]
use crate::{
    ir::{
        builder::IRBuilder,
        types::{Branch, Env, Operation, TIRFunction, Terminator, VirtualRegister},
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

    let merge_block = builder.init_block();
    let mut incoming = vec![];
    let mut next_block = builder.current();
    let len = branches.len();

    for (i, (cond_opt, body)) in branches.into_iter().enumerate() {
        let is_last = i == len - 1;

        if let Some(cond_expr) = cond_opt {
            let then_block = builder.init_block();
            let else_block = if is_last {
                // last condition with no explicit else
                //
                // e.g. `if <cond> { ... } else if <cond> { ... }`
                merge_block
            } else {
                builder.init_block()
            };
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
                incoming.push(vals.clone());

                builder.terminate(Terminator::Branch {
                    target: merge_block,
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
                incoming.push(vals.clone());

                builder.terminate(Terminator::Branch {
                    target: merge_block,
                    params: vals,
                });
            }
        }
    }

    // If no branches reach merge
    if incoming.is_empty() {
        return env;
    }

    let params = (0..vars.len()).map(|_| builder.value()).collect::<Vec<_>>();
    builder.switch_to(merge_block);
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
