use super::ast::{ArithmeticOpcode, CondBlock, Expr, ListItem, Opcode, Prog, Stmt};
use super::value::Value;
use std::collections::HashMap;

pub type Env<'a> = HashMap<&'a str, Value<'a>>;

pub fn run_prog<'a>(env: &mut Env<'a>, Prog::Body(stmts): &'a Prog) -> Result<(), String> {
    run_block(env, stmts)
}

fn run_block<'a>(env: &mut Env<'a>, stmts: &'a Vec<Stmt>) -> Result<(), String> {
    for stmt in stmts {
        run_stmt(env, stmt)?;
    }
    Ok(())
}

fn run_stmt<'a>(env: &mut Env<'a>, stmt: &'a Stmt) -> Result<(), String> {
    match stmt {
        Stmt::Expr(expr) => {
            run_expr(env, expr)?;
        }
        Stmt::Assign { lhs, rhs } => {
            match lhs {
                Expr::Ident(name) => {
                    let value = run_expr(env, rhs)?;
                    env.insert(name, value);
                }
                Expr::List(lhs) => {
                    // don't let zero sized arrays to be destructured
                    if lhs.len() == 0 {
                        return Err(format!("cannot destructure empty list"));
                    }
                    let lhs_count: u32 = lhs
                        .iter()
                        .filter_map(|li| match li.expr {
                            Expr::Ident(_) | Expr::Underscore => Some(1),
                            _ => None,
                        })
                        .sum();
                    // only underscores and identifiers are permitted on the left hand side
                    if lhs_count > lhs.len() as u32 {
                        return Err(format!("list destructuring not possible: detected non identifier expression on the left hand side of destructuring"));
                    }
                    let lhs_rest_count: u32 = lhs
                        .iter()
                        .filter_map(|f| if f.is_spread { Some(1) } else { None })
                        .sum();
                    if lhs_rest_count > 1 {
                        return Err(format!("list destructuring not possible: expected at most 1 identifier with rest operator, found '{lhs_rest_count}'"));
                    }
                    if let Expr::List(rhs) = rhs {
                        // don't let zero sized arrays to be destructured
                        if rhs.len() == 0 {
                            return Err(format!("cannot destructure empty list"));
                        }
                        // underscores are not allowed on the right hand side
                        if rhs.iter().any(|f| {
                            if let Expr::Underscore = f.expr {
                                true
                            } else {
                                false
                            }
                        }) {
                            return Err(format!("list destructuring not possible: '_' is not allowed in the list to be destructured"));
                        }
                        let rhs_count: u32 = rhs
                            .iter()
                            .filter_map(|ri| match &ri.expr {
                                Expr::Ident(n) => {
                                    if ri.is_spread {
                                        match env.get(n) {
                                            Some(v) => {
                                                if let Value::List(vl) = v {
                                                    Some(vl.len() as u32)
                                                } else {
                                                    Some(1)
                                                }
                                            }
                                            None => None,
                                        }
                                    } else {
                                        Some(1)
                                    }
                                }
                                Expr::Int(_)
                                | Expr::StrLiteral(_)
                                | Expr::Call { func: _, args: _ }
                                | Expr::Op {
                                    lhs: _,
                                    opcode: _,
                                    rhs: _,
                                } => Some(1),
                                Expr::List(rl) => {
                                    if ri.is_spread {
                                        Some(rl.len() as u32)
                                    } else {
                                        Some(1)
                                    }
                                }
                                _ => None,
                            })
                            .sum();
                        // do not allow non-exhaustive list destructuring
                        if rhs_count > lhs_count && lhs_rest_count == 0 {
                            return Err(format!("list destructuring not possible: not enough coverage of the list on the left hand side"));
                        }
                        // calculate right hand side
                        let mut rhs_vals = Vec::with_capacity(rhs_count as usize);
                        for item in rhs {
                            if item.is_spread {
                                let val = run_expr(env, &item.expr)?;
                                match val {
                                    Value::List(mut lst_val) => rhs_vals.append(&mut lst_val),
                                    _ => return Err(format!("cannot spread non-list value")),
                                }
                            } else {
                                let val = run_expr(env, &item.expr)?;
                                rhs_vals.push(val);
                            }
                        }
                        // assign right hand side values into corresponding left hand side vars
                        let rhs_count = rhs_vals.len();
                        let rhs_vals = rhs_vals.into_iter();
                        destructure_list(env, lhs, rhs_vals, rhs_count)?;
                    } else if let Expr::Ident(rhs) = rhs {
                        let rhs_val = env
                            .get(rhs)
                            .cloned()
                            .ok_or(&format!("identifier '{rhs}' not defined"))?;
                        if let Value::List(rhs_vals) = rhs_val {
                            let rhs_count = rhs_vals.len();
                            // do not allow non-exhaustive list destructuring
                            if rhs_vals.len() > lhs_count as usize && lhs_rest_count == 0 {
                                return Err(format!("list destructuring not possible: not enough coverage of the list on the left hand side"));
                            }
                            destructure_list(env, lhs, rhs_vals.into_iter(), rhs_count)?;
                        } else {
                            return Err(format!("type mismatch on assignment: list destructuring can only be done on a list"));
                        }
                    } else {
                        return Err(format!("type mismatch on assignment: list destructuring can only be done on a list"));
                    }
                }
                Expr::Underscore => {}
                Expr::Call { func: _, args: _ } => {
                    return Err(format!("cannot assign to a function call"))
                }
                Expr::StrLiteral(_) => return Err(format!("cannot assign to a string literal")),
                Expr::Int(_) => return Err(format!("cannot assign to an integer literal")),
                Expr::Op {
                    lhs: _,
                    rhs: _,
                    opcode: _,
                } => return Err(format!("cannot assign to an arithmetic operation")),
            }
        }
        Stmt::ArithmeticAssign { name, rhs, opcode } => {
            let value = run_expr(env, rhs)?;
            let old = env.get(name).ok_or(format!("'{name}' is not defined"))?;
            match opcode {
                &ArithmeticOpcode::AddAssign => {
                    let new = (old + &value)?;
                    env.insert(name, new);
                }
                &ArithmeticOpcode::SubAssign => {
                    let new = (old - &value)?;
                    env.insert(name, new);
                }
                &ArithmeticOpcode::DivAssign => {
                    let new = (old / &value)?;
                    env.insert(name, new);
                }
                &ArithmeticOpcode::MulAssign => {
                    let new = (old * &value)?;
                    env.insert(name, new);
                }
                &ArithmeticOpcode::ModAssign => {
                    let new = (old % &value)?;
                    env.insert(name, new);
                }
            }
        }
        Stmt::IfElse {
            if_block: CondBlock { cond, stmts },
            else_if_blocks,
            else_block,
        } => {
            if let Ok(Value::Boolean(v)) = run_expr(env, cond) {
                if v {
                    return run_block(env, stmts);
                }
                if else_if_blocks.len() > 0 {
                    for CondBlock { cond, stmts } in else_if_blocks {
                        if let Ok(Value::Boolean(v)) = run_expr(env, cond) {
                            if v {
                                return run_block(env, stmts);
                            }
                        } else {
                            return Err(format!("expected boolean expression in if statement: '{cond:?}' is not a boolean expression"));
                        }
                    }
                }
                if let Some(stmts) = else_block {
                    return run_block(env, stmts);
                }
            } else {
                return Err(format!("expected boolean expression in if statement: '{cond:?}' is not a boolean expression"));
            }
        }
        Stmt::While(CondBlock { cond, stmts }) => loop {
            if let Ok(Value::Boolean(v)) = run_expr(env, cond) {
                if v {
                    run_block(env, stmts)?;
                } else {
                    break;
                }
            } else {
                return Err(format!("expected boolean expression in while statement: '{cond:?}' is not a boolean expression"));
            }
        },
    }

    Ok(())
}

fn run_expr<'a>(env: &mut Env<'a>, expr: &Expr<'a>) -> Result<Value<'a>, String> {
    match &expr {
        Expr::Ident(name) => env
            .get(name)
            .cloned()
            .ok_or(format!("'{name}' is not defined")),
        Expr::Int(v) => Ok(Value::Int(*v)),
        Expr::StrLiteral(v) => Ok(Value::StrLiteral(v)),
        Expr::Underscore => Ok(Value::Unit),
        Expr::List(items) => {
            let mut vals = Vec::with_capacity(items.len());
            for item in items {
                if item.is_spread {
                    match run_expr(env, &item.expr)? {
                        Value::List(mut items) => vals.append(&mut items),
                        _ => return Err(format!("'..' operator can only be applied to lists")),
                    }
                } else {
                    vals.push(run_expr(env, &item.expr)?)
                }
            }
            Ok(Value::List(vals))
        }
        Expr::Call { func, args } => {
            let mut vals = Vec::with_capacity(args.len());
            for arg in args {
                match run_expr(env, arg) {
                    Ok(r) => vals.push(r),
                    Err(e) => return Err(e),
                }
            }
            let v = env.get(func).ok_or(format!("'{}' is not defined", func))?;
            if let Value::Func(f) = v {
                f(vals)
            } else {
                Err(format!("'{}' is not a function", func))
            }
        }
        Expr::Op { lhs, rhs, opcode } => {
            use Opcode::*;
            let lhs = run_expr(env, lhs)?;
            let rhs = run_expr(env, rhs)?;
            if std::mem::discriminant(&lhs) != std::mem::discriminant(&rhs) {
                return Err(String::from(
                    "type mismatch: lhs and rhs of the expression are not same type",
                ));
            }
            match *opcode {
                Eq => Ok(Value::Boolean(lhs == rhs)),
                Neq => Ok(Value::Boolean(lhs != rhs)),
                Gt => Ok(Value::Boolean(lhs > rhs)),
                Lt => Ok(Value::Boolean(lhs < rhs)),
                Gte => Ok(Value::Boolean(lhs >= rhs)),
                Lte => Ok(Value::Boolean(lhs <= rhs)),
                Add => &lhs + &rhs,
                Sub => &lhs - &rhs,
                Div => &lhs / &rhs,
                Mul => &lhs * &rhs,
                Mod => &lhs % &rhs,
            }
        }
    }
}

fn destructure_list<'a, T>(
    env: &mut Env<'a>,
    lhs: &Vec<ListItem<'a>>,
    mut rhs_vals: T,
    rhs_count: usize,
) -> Result<(), String>
where
    T: Iterator<Item = Value<'a>>,
{
    for (index, item) in lhs.iter().enumerate() {
        if item.is_spread {
            // it is safe to assume that this is the only rest op
            // because we made the checks beforehand
            let items_to_take = (rhs_count - index) - (lhs.len() - index) + 1;
            if let Expr::Ident(name) = item.expr {
                let mut vals = Vec::with_capacity(items_to_take);
                for _ in 0..items_to_take {
                    vals.push(rhs_vals.next().unwrap());
                }
                env.insert(name, Value::List(vals));
            } else {
                return Err(format!(
                    "dev error: expected identifier, found '{:?}'",
                    item.expr
                ));
            }
        } else if let Expr::Underscore = item.expr {
            rhs_vals.next();
        } else if let Expr::Ident(name) = item.expr {
            env.insert(name, rhs_vals.next().unwrap());
        } else {
            return Err(format!(
                "dev error: expected identifier or underscore '_', found '{:?}'",
                item.expr
            ));
        }
    }
    Ok(())
}
