use crate::err::RuntimeError;
use crate::lex::Punctuation;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PscObject {
    IntT(i64),
    FloatT(f64),
    StringT(String),
    BoolT(bool),
}

#[derive(Debug)]
pub enum Stmt {
    Assign(Assign),
    Input(Input),
    Output(Output),
    If(If),
    While(While),
    Until(Until),
    For(For),
}

impl Stmt {
    pub fn eval(stmt: &Self, vars: &mut HashMap<String, PscObject>) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Assign(assign) => {
                let res = Expr::eval(&assign.expr, vars)?;
                vars.insert(assign.ident.to_string(), res);
            }

            Stmt::Output(output) => {
                let res = Expr::eval(&output.expr, vars)?;

                match res {
                    PscObject::IntT(x) => println!("{}", x),
                    PscObject::FloatT(x) => println!("{}", x),
                    PscObject::StringT(x) => println!("{}", x),
                    PscObject::BoolT(x) => println!("{}", x),
                };
            }

            Stmt::Input(input) => {
                let mut buffer = String::new();
                if let Err(e) = std::io::stdin().read_line(&mut buffer) {
                    return Err(RuntimeError { msg: e.to_string() });
                }

                let striped_buffer = buffer.trim();

                if let Ok(x) = striped_buffer.parse::<bool>() {
                    vars.insert(input.ident.to_string(), PscObject::BoolT(x));
                } else if let Ok(x) = striped_buffer.parse::<i64>() {
                    vars.insert(input.ident.to_string(), PscObject::IntT(x));
                } else if let Ok(x) = striped_buffer.parse::<f64>() {
                    vars.insert(input.ident.to_string(), PscObject::FloatT(x));
                } else {
                    vars.insert(input.ident.to_string(), PscObject::StringT(buffer));
                }
            }

            Stmt::If(if_stmt) => {
                for (cond, stmts) in &if_stmt.branches {
                    if let PscObject::BoolT(b) = Expr::eval(&cond, vars)? {
                        if b {
                            for stmt in stmts {
                                Stmt::eval(&stmt, vars)?;
                            }
                            break;
                        }
                    } else {
                        return Err(RuntimeError { msg: "If expression not bool type".into()  });
                    }
                }
            }

            Stmt::While(while_stmt) => {
                loop {
                    if let PscObject::BoolT(b) = Expr::eval(&while_stmt.cond, vars)? {
                        if b {
                            for stmt in &while_stmt.stmts {
                                Stmt::eval(&stmt, vars)?;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            Stmt::Until(until_stmt) => {
                loop {
                    if let PscObject::BoolT(b) = Expr::eval(&until_stmt.cond, vars)? {
                        if !b {
                            for stmt in &until_stmt.stmts {
                                Stmt::eval(&stmt, vars)?;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            Stmt::For(for_stmt) => {
                let start = Expr::eval(&for_stmt.start, vars)?;
                let end = Expr::eval(&for_stmt.end, vars)?;
                vars.insert(for_stmt.name.clone(), PscObject::IntT(0));

                if let (PscObject::IntT(s), PscObject::IntT(e)) = (start, end) {
                    for i in s..=e {
                        if let Some(x) = vars.get_mut(&for_stmt.name) {
                            *x = PscObject::IntT(i);

                            for stmt in &for_stmt.stmts {
                                Stmt::eval(&stmt, vars)?;
                            } 
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct If {
    pub branches: Vec<(Expr, Vec<Stmt>)>
}

#[derive(Debug)]
pub struct While {
    pub cond: Expr,
    pub stmts: Vec<Stmt>
}

#[derive(Debug)]
pub struct Until {
    pub cond: Expr,
    pub stmts: Vec<Stmt>
}

#[derive(Debug)]
pub struct For {
    pub name: String,
    pub start: Expr,
    pub end: Expr,
    pub stmts: Vec<Stmt>
}

#[derive(Debug)]
pub struct Assign {
    pub ident: String,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Input {
    pub ident: String,
}

#[derive(Debug)]
pub struct Output {
    pub expr: Expr,
}

#[derive(Debug)]
pub struct BinOp {
    pub left: Expr,
    pub right: Expr,
    pub op: Punctuation,
}

#[derive(Debug)]
pub enum Expr {
    BinOp(Box<BinOp>),
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StrLit(String),
    Ident(String),
}

impl Expr {
    fn eval(expr: &Self, vars: &mut HashMap<String, PscObject>) -> Result<PscObject, RuntimeError> {
        match expr {
            Expr::IntLit(x) => Ok(PscObject::IntT(*x)),
            Expr::FloatLit(x) => Ok(PscObject::FloatT(*x)),
            Expr::StrLit(x) => Ok(PscObject::StringT(x.to_string())),
            Expr::BoolLit(x) => Ok(PscObject::BoolT(*x)),
            Expr::Ident(x) => {
                if let Some(val) = vars.get(x) {
                    Ok(val.clone())
                } else {
                    Err(RuntimeError {
                        msg: format!("Unknow identifier: {}", x),
                    })
                }
            }
            Expr::BinOp(bin_op) => {
                let left = Expr::eval(&bin_op.left, vars)?;
                let right = Expr::eval(&bin_op.right, vars)?;

                let ret = match bin_op.op {
                    Punctuation::Plus => match (left, right) {
                        (PscObject::IntT(l), PscObject::IntT(r)) => PscObject::IntT(l + r),
                        (PscObject::FloatT(l), PscObject::FloatT(r)) => PscObject::FloatT(l + r),
                        (PscObject::IntT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT(l as f64 + r)
                        }
                        (PscObject::FloatT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT(l + r as f64)
                        }

                        (PscObject::StringT(l), PscObject::StringT(r)) => {
                            PscObject::StringT(format!("{}{}", l, r))
                        }

                        _ => {
                            return Err(RuntimeError {
                                msg: "Mismatched types".into(),
                            })
                        }
                    },

                    Punctuation::Minus => match (left, right) {
                        (PscObject::IntT(l), PscObject::IntT(r)) => PscObject::IntT(l - r),
                        (PscObject::FloatT(l), PscObject::FloatT(r)) => PscObject::FloatT(l - r),
                        (PscObject::IntT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT(l as f64 - r)
                        }
                        (PscObject::FloatT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT(l - r as f64)
                        }

                        _ => {
                            return Err(RuntimeError {
                                msg: "Mismatched types".into(),
                            })
                        }
                    },

                    Punctuation::Mul => match (left, right) {
                        (PscObject::IntT(l), PscObject::IntT(r)) => PscObject::IntT(l * r),
                        (PscObject::FloatT(l), PscObject::FloatT(r)) => PscObject::FloatT(l * r),
                        (PscObject::IntT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT(l as f64 * r)
                        }
                        (PscObject::FloatT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT(l * r as f64)
                        }

                        _ => {
                            return Err(RuntimeError {
                                msg: "Mismatched types".into(),
                            })
                        }
                    },

                    Punctuation::Div => match (left, right) {
                        (PscObject::IntT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT(l as f64 / r as f64)
                        }
                        (PscObject::FloatT(l), PscObject::FloatT(r)) => PscObject::FloatT(l / r),
                        (PscObject::IntT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT(l as f64 / r)
                        }
                        (PscObject::FloatT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT(l / r as f64)
                        }

                        _ => {
                            return Err(RuntimeError {
                                msg: "Mismatched types".into(),
                            })
                        }
                    },

                    Punctuation::FloorDiv => match (left, right) {
                        (PscObject::IntT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT((l as f64 / r as f64).floor())
                        }
                        (PscObject::FloatT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT((l / r).floor())
                        }
                        (PscObject::IntT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT((l as f64 / r).floor())
                        }
                        (PscObject::FloatT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT((l / r as f64).floor())
                        }

                        _ => {
                            return Err(RuntimeError {
                                msg: "Mismatched types".into(),
                            })
                        }
                    },

                    Punctuation::Mod => match (left, right) {
                        (PscObject::IntT(l), PscObject::IntT(r)) => PscObject::IntT(l % r),
                        (PscObject::FloatT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT((l % r).floor())
                        }
                        (PscObject::IntT(l), PscObject::FloatT(r)) => {
                            PscObject::FloatT((l as f64 % r).floor())
                        }
                        (PscObject::FloatT(l), PscObject::IntT(r)) => {
                            PscObject::FloatT((l % r as f64).floor())
                        }

                        _ => {
                            return Err(RuntimeError {
                                msg: "Mismatched types".into(),
                            })
                        }
                    }

                    Punctuation::GE | Punctuation::LE | Punctuation::GT | Punctuation::LT => {
                        let (a, b, c) = match (left, right) {
                            (PscObject::IntT(l), PscObject::IntT(r)) => (l > r, l == r, l < r),
                            (PscObject::FloatT(l), PscObject::FloatT(r)) => (l > r, l == r, l < r),
                            (PscObject::IntT(l), PscObject::FloatT(r)) => {
                                let l = l as f64;
                                (l > r, l == r, l < r)
                            }
                            (PscObject::FloatT(l), PscObject::IntT(r)) => {
                                let r = r as f64;
                                (l > r, l == r, l < r)
                            }
                            _ => todo!(),
                        };

                        match bin_op.op {
                            Punctuation::GE => PscObject::BoolT(a || b),
                            Punctuation::LE => PscObject::BoolT(c || b),
                            Punctuation::GT => PscObject::BoolT(a),
                            Punctuation::LT => PscObject::BoolT(c),
                            _ => unreachable!(),
                        }
                    }

                    Punctuation::Equals => match (left, right) {
                        (PscObject::IntT(l), PscObject::IntT(r)) => PscObject::BoolT(l == r),
                        (PscObject::FloatT(l), PscObject::FloatT(r)) => PscObject::BoolT(l == r),
                        (PscObject::StringT(l), PscObject::StringT(r)) => PscObject::BoolT(l == r),
                        (PscObject::BoolT(l), PscObject::BoolT(r)) => PscObject::BoolT(l == r),

                        (PscObject::IntT(l), PscObject::FloatT(r)) => {
                            PscObject::BoolT(l as f64 == r)
                        }
                        (PscObject::FloatT(l), PscObject::IntT(r)) => {
                            PscObject::BoolT(l == r as f64)
                        }

                        _ => {
                            return Err(RuntimeError {
                                msg: "Mismatched types".into(),
                            })
                        }
                    },

                    _ => todo!(),
                };

                Ok(ret)
            }
        }
    }
}
