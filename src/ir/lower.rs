use crate::{
    ir::{
        builder::IRBuilder,
        types::{Branch, Env, Operation, PendingEdge, TIRFunction, Terminator, VirtualRegister},
    },
    parser::BinOp,
    resolver::{
        ResolvedElseClause, ResolvedExpression, ResolvedFunction, ResolvedProgram,
        ResolvedStatement, ResolvedTerm, SymbolId,
    },
};

pub fn lower_program(program: ResolvedProgram) -> Vec<TIRFunction> {
    let mut functions = Vec::new();
    if let Some(main) = program.main {
        functions.push(lower_function(main));
    }
    for function in program.functions {
        functions.push(lower_function(function));
    }

    functions
}

fn lower_function(function: ResolvedFunction) -> TIRFunction {
    let mut builder = IRBuilder::new();
    let mut env = Env::new();

    let params = function
        .args
        .iter()
        .map(|arg| {
            let value = builder.value();
            env.insert(arg.symbol.clone(), value);
            value
        })
        .collect::<Vec<_>>();
    builder.current_mut().params = params.clone();

    lower_scope(function.body, env, &mut builder);

    // handle implicit returns for void functions
    let last = builder
        .blocks
        .last_mut()
        .expect("must have at least one block");
    if last.terminator == Terminator::Void {
        last.terminator = Terminator::Return(VirtualRegister(0));
    }

    TIRFunction {
        name: function.name,
        params,
        blocks: builder.to_blocks(),
    }
}

fn lower_scope(
    scope: Vec<ResolvedStatement>,
    mut env: Env,
    builder: &mut IRBuilder,
) -> Option<Env> {
    for statement in scope {
        env = lower_statement(statement, env.clone(), builder)?;
    }

    Some(env)
}

fn lower_statement(
    statement: ResolvedStatement,
    mut env: Env,
    builder: &mut IRBuilder,
) -> Option<Env> {
    match statement {
        ResolvedStatement::Exit(expr) => {
            let v = lower_expr(expr, &env, builder);
            builder.terminate(Terminator::Exit(v));
            None
        }
        ResolvedStatement::Let { symbol, expr, .. } => {
            let v = lower_expr(expr, &env, builder);
            env.insert(symbol, v);
            Some(env)
        }
        ResolvedStatement::Assignment { symbol, expr } => {
            let v = lower_expr(expr, &env, builder);
            env.insert(symbol, v);
            Some(env)
        }
        ResolvedStatement::If { cond, then, els } => Some(lower_if(cond, then, els, env, builder)),
        ResolvedStatement::While { cond, body } => Some(lower_while(cond, body, env, builder)),
        ResolvedStatement::FunctionCall { function, args } => {
            let arg_vals = args
                .into_iter()
                .map(|a| lower_expr(a, &env, builder))
                .collect();

            let _result = builder.emit(Operation::Call(function, arg_vals));
            Some(env)
        }
        ResolvedStatement::Return(expr) => {
            let v = lower_expr(expr, &env, builder);
            builder.terminate(Terminator::Return(v));
            None
        }
    }
}

fn lower_if(
    cond: ResolvedExpression,
    then_body: Vec<ResolvedStatement>,
    else_clause: Option<ResolvedElseClause>,
    env: Env,
    builder: &mut IRBuilder,
) -> Env {
    let branches = flatten_if_chain(cond, then_body, else_clause);
    let vars: Vec<SymbolId> = env.keys().cloned().collect();

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
                    from: builder.current(),
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
                    from: builder.current(),
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
    cond: ResolvedExpression,
    then: Vec<ResolvedStatement>,
    els: Option<ResolvedElseClause>,
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

fn lower_while(
    cond: ResolvedExpression,
    body: Vec<ResolvedStatement>,
    env: Env,
    builder: &mut IRBuilder,
) -> Env {
    let vars: Vec<SymbolId> = env.keys().cloned().collect();

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

    let body_params = (0..vars.len()).map(|_| builder.value()).collect::<Vec<_>>();
    builder.switch_to(body_block);
    builder.current_mut().params = body_params.clone();
    let mut body_env = Env::new();
    for (i, var) in vars.iter().enumerate() {
        body_env.insert(var.clone(), body_params[i]);
    }
    let body_env = lower_scope(body, body_env, builder).unwrap();
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

fn lower_expr(expr: ResolvedExpression, env: &Env, builder: &mut IRBuilder) -> VirtualRegister {
    match expr {
        ResolvedExpression::BinaryExpr(lhs, rhs, op) => {
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
        ResolvedExpression::Term(term) => match term {
            ResolvedTerm::Identifier(sym) => {
                env.get(&sym).expect("identifier must exist").to_owned()
            }
            ResolvedTerm::IntLit(n) => builder.emit(Operation::ConstInt(n)),
            ResolvedTerm::Bool(b) => builder.emit(Operation::ConstBool(b)),
        },
        ResolvedExpression::FunctionCall { function, args } => {
            let args = args
                .into_iter()
                .map(|a| lower_expr(a, env, builder))
                .collect();

            builder.emit(Operation::Call(function, args))
        }
    }
}
