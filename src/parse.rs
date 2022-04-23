use crate::err::ParseError;
use crate::eval::*;
use crate::lex::{Keyword, LexerToken, Punctuation};

type TokenStream<'a> = std::iter::Peekable<std::slice::Iter<'a, LexerToken>>;

pub fn parse(tokens: Vec<LexerToken>) -> Result<Vec<Stmt>, ParseError> {
    let mut ret: Vec<Stmt> = vec![];
    let mut it = tokens.iter().peekable();

    while let Some(_) = it.peek() {
        ret.push(parse_stmt(&mut it)?);
    }

    Ok(ret)
}

fn parse_stmt(tokens: &mut TokenStream) -> Result<Stmt, ParseError> {
    match tokens.peek() {
        Some(&LexerToken::Identifier(ident)) => {
            tokens.next();

            if tokens.next() == Some(&LexerToken::Punctuation(Punctuation::Assign)) {
                let left = parse_atom(tokens)?;

                return Ok(Stmt::Assign(Assign {
                    ident: ident.to_string(),
                    expr: parse_bin_op(tokens, left, 0)?,
                }));
            }
        }

        Some(&LexerToken::Keyword(Keyword::Input)) => {
            tokens.next();

            if let Some(&LexerToken::Identifier(ident)) = tokens.peek() {
                tokens.next();

                return Ok(Stmt::Input(Input {
                    ident: ident.to_string(),
                }));
            } else {
                return Err(ParseError {
                    msg: "Failed to parse input stmt".into(),
                });
            }
        }

        Some(&LexerToken::Keyword(Keyword::Output)) => {
            tokens.next();

            let left = parse_atom(tokens)?;

            return Ok(Stmt::Output(Output {
                expr: parse_bin_op(tokens, left, 0)?,
            }));
        }

        Some(&LexerToken::Keyword(Keyword::If)) => {
            tokens.next();

            let left = parse_atom(tokens)?;
            let mut branches: Vec<(Expr, Vec<Stmt>)> = vec![(parse_bin_op(tokens, left, 0)?, Vec::new())];

            if tokens.next() != Some(&LexerToken::Keyword(Keyword::Then)) {
                return Err(ParseError {
                    msg: "Failed to parse if stmt".into(),
                });
            }

            loop {
                match tokens.peek() {
                    Some(&LexerToken::Keyword(Keyword::Else)) => {
                        tokens.next();

                        let cond = {
                            if tokens.peek() == Some(&&LexerToken::Keyword(Keyword::If)) {
                                tokens.next();

                                let left = parse_atom(tokens)?;
                                parse_bin_op(tokens, left, 0)?
                            } else {
                                Expr::BoolLit(true)
                            }
                        };

                        branches.push((cond, Vec::new()));
                    }
                    Some(&LexerToken::Keyword(Keyword::End)) => {
                        tokens.next();

                        if tokens.next() != Some(&&LexerToken::Keyword(Keyword::If)) {
                            return Err(ParseError {
                                msg: "Failed to parse if stmt".into(),
                            });
                        }

                        return Ok(Stmt::If(If {
                            branches
                        }));
                    }

                    _ => {
                        let len = branches.len();
                        branches[len - 1].1.push(parse_stmt(tokens)?);
                    }
                }
            }
        }

        Some(&LexerToken::Keyword(Keyword::Loop)) => {
            tokens.next();

            match tokens.next() {
                Some(LexerToken::Keyword(Keyword::While)) => {
                    let left = parse_atom(tokens)?;
                    let cond = parse_bin_op(tokens, left, 0)?;
                    let mut stmts: Vec<Stmt> = Vec::new();

                    while tokens.peek() != Some(&&LexerToken::Keyword(Keyword::End)) {
                        stmts.push(parse_stmt(tokens)?);
                    }
                    tokens.next();

                    if tokens.next() != Some(&&LexerToken::Keyword(Keyword::Loop)) {
                        return Err(ParseError {
                            msg: "Failed to parse while stmt".into(),
                        });
                    }

                    return Ok(Stmt::While(While {
                        cond,
                        stmts,
                    }));
                }

                Some(LexerToken::Keyword(Keyword::Until)) => {
                    let left = parse_atom(tokens)?;
                    let cond = parse_bin_op(tokens, left, 0)?;
                    let mut stmts: Vec<Stmt> = Vec::new();

                    while tokens.peek() != Some(&&LexerToken::Keyword(Keyword::End)) {
                        stmts.push(parse_stmt(tokens)?);
                    }
                    tokens.next();

                    if tokens.next() != Some(&&LexerToken::Keyword(Keyword::Loop)) {
                        return Err(ParseError {
                            msg: "Failed to parse while stmt".into(),
                        });
                    }

                    return Ok(Stmt::Until(Until {
                        cond,
                        stmts,
                    }));
                }

                Some(LexerToken::Identifier(name)) => {
                    if tokens.next() != Some(&LexerToken::Keyword(Keyword::From)) {
                        return Err(ParseError {
                            msg: "Failed to parse 'from' in for stmt".into(),
                        });
                    }

                    let start = {
                        let left = parse_atom(tokens)?;
                        parse_bin_op(tokens, left, 0)?
                    };

                    if tokens.next() != Some(&LexerToken::Keyword(Keyword::To)) {
                        return Err(ParseError {
                            msg: "Failed to parse 'to' in for stmt".into(),
                        });
                    }

                    let end = {
                        let left = parse_atom(tokens)?;
                        parse_bin_op(tokens, left, 0)?
                    };

                    let mut stmts: Vec<Stmt> = Vec::new();
                    while tokens.peek() != Some(&&LexerToken::Keyword(Keyword::End)) {
                        stmts.push(parse_stmt(tokens)?);
                    }
                    tokens.next();

                    if tokens.next() != Some(&&LexerToken::Keyword(Keyword::Loop)) {
                        return Err(ParseError {
                            msg: "Failed to parse stmt".into(),
                        });
                    }

                    return Ok(Stmt::For(For {
                        name: name.clone(),
                        start,
                        end,
                        stmts,
                    }));
                }
                _ => todo!()
            }
        }

        _ => {
            return Err(ParseError {
                msg: "Failed to parse stmt".into(),
            });
        }
    };

    unreachable!();
}

fn parse_atom(tokens: &mut TokenStream) -> Result<Expr, ParseError> {
    match tokens.peek() {
        Some(&LexerToken::IntLit(x)) => {
            tokens.next();
            return Ok(Expr::IntLit(*x));
        }

        Some(&LexerToken::FloatLit(x)) => {
            tokens.next();
            return Ok(Expr::FloatLit(*x));
        }

        Some(&LexerToken::BoolLit(x)) => {
            tokens.next();
            return Ok(Expr::BoolLit(*x));
        }

        Some(&LexerToken::StrLit(x)) => {
            tokens.next();
            return Ok(Expr::StrLit(x.to_string()));
        }

        Some(&LexerToken::Identifier(ident)) => {
            tokens.next();
            return Ok(Expr::Ident(ident.to_string()));
        }

        _ => {
            return Err(ParseError {
                msg: "Failed to parse atom".into(),
            });
        }
    }
}

fn parse_bin_op(tokens: &mut TokenStream, left: Expr, precedence: u32) -> Result<Expr, ParseError> {
    match tokens.peek() {
        Some(&LexerToken::Punctuation(op)) if op != &Punctuation::Assign => {
            let new_precedence = op.precedence();

            if new_precedence >= precedence {
                tokens.next();
                let next_atom = parse_atom(tokens)?;

                let ret = Expr::BinOp(Box::new(BinOp {
                    left,
                    right: parse_bin_op(tokens, next_atom, new_precedence)?,
                    op: op.clone(),
                }));

                return Ok(parse_bin_op(tokens, ret, precedence)?);
            } else {
                return Ok(left);
            }
        }

        _ => {
            return Ok(left);
        }
    }
}
