use crate::err::ParseError;

#[derive(Debug, PartialEq)]
pub enum LexerToken {
    Keyword(Keyword),
    Punctuation(Punctuation),
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StrLit(String),
    Identifier(String),
}

impl LexerToken {
    pub fn from_identifier(keyword: &str) -> Option<LexerToken> {
        match keyword {
            "loop" => Some(LexerToken::Keyword(Keyword::Loop)),
            "while" => Some(LexerToken::Keyword(Keyword::While)),
            "from" => Some(LexerToken::Keyword(Keyword::From)),
            "to" => Some(LexerToken::Keyword(Keyword::To)),
            "until" => Some(LexerToken::Keyword(Keyword::Until)),
            "if" => Some(LexerToken::Keyword(Keyword::If)),
            "then" => Some(LexerToken::Keyword(Keyword::Then)),
            "else" => Some(LexerToken::Keyword(Keyword::Else)),
            "end" => Some(LexerToken::Keyword(Keyword::End)),
            "input" => Some(LexerToken::Keyword(Keyword::Input)),
            "output" => Some(LexerToken::Keyword(Keyword::Output)),

            "mod" => Some(LexerToken::Punctuation(Punctuation::Mod)),
            "div" => Some(LexerToken::Punctuation(Punctuation::FloorDiv)),

            "true" => Some(LexerToken::BoolLit(true)),
            "false" => Some(LexerToken::BoolLit(false)),

            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Loop,
    While,
    From,
    To,
    Until,
    If,
    Then,
    Else,
    End,
    Input,
    Output,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Punctuation {
    Plus,
    Minus,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Assign,
    Equals,
    GT,
    LT,
    GE,
    LE,
}

impl Punctuation {
    pub fn precedence(&self) -> u32 {
        match *self {
            Punctuation::Assign => 0,
            Punctuation::Equals => 1,
            Punctuation::GT => 1,
            Punctuation::LT => 1,
            Punctuation::GE => 1,
            Punctuation::LE => 1,
            Punctuation::Plus => 2,
            Punctuation::Minus => 2,
            Punctuation::Mul => 3,
            Punctuation::Div => 3,
            Punctuation::FloorDiv => 3,
            Punctuation::Mod => 3,
        }
    }
}

pub fn lex(prog: &str) -> Result<Vec<LexerToken>, ParseError> {
    let mut ret: Vec<LexerToken> = vec![];
    let mut it = prog.chars().peekable();

    while let Some(c) = it.peek() {
        match c {
            ' ' | '\n' | '\t' => {
                it.next();
            }

            c if c.is_digit(10) => {
                let mut buf = String::new();
                let mut is_float = false;

                while let Some(c) = it.next() {
                    match c {
                        c if c.is_digit(10) => buf.push(c),
                        '.' if !is_float => {
                            buf.push('.');
                            is_float = true;
                        }

                        '.' if is_float => {
                            return Err(ParseError {
                                msg: "Malformed float literal".into(),
                            })
                        }

                        _ => break,
                    }
                }

                if is_float {
                    let x: f64 = match buf.parse() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(ParseError {
                                msg: "Failed to parse float literal".into(),
                            })
                        }
                    };

                    ret.push(LexerToken::FloatLit(x));
                } else {
                    let x: i64 = match buf.parse() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(ParseError {
                                msg: "Failed to parse int literal".into(),
                            })
                        }
                    };

                    ret.push(LexerToken::IntLit(x));
                }
            }

            '\"' => {
                let mut buf = String::new();
                it.next();

                while let Some(c) = it.next() {
                    match c {
                        '\"' => break,
                        _ => buf.push(c),
                    }
                }

                if let None = it.peek() {
                    return Err(ParseError {
                        msg: "Failed to parse string literal".into(),
                    });
                }

                ret.push(LexerToken::StrLit(buf));
            }

            c if c.is_ascii_alphabetic() => {
                let mut buf = String::new();

                while let Some(c) = it.next() {
                    match c {
                        c if c.is_ascii_alphabetic() => buf.push(c),
                        '_' => buf.push('_'),
                        _ => break,
                    }
                }

                if let Some(tok) = LexerToken::from_identifier(&buf.to_lowercase()) {
                    ret.push(tok);
                } else {
                    for c in buf.chars() {
                        if !c.is_uppercase() && c != '_' {
                            return Err(ParseError {
                                msg: format!("Invalid identifier: {}", &buf),
                            });
                        }
                    }

                    ret.push(LexerToken::Identifier(buf.clone()))
                }
            }

            c if c.is_ascii_punctuation() => {
                let c = match it.next() {
                    Some(x) => x,
                    None => {
                        return Err(ParseError {
                            msg: "Unexpected EOF".into(),
                        })
                    }
                };

                let tok = match c {
                    '+' => LexerToken::Punctuation(Punctuation::Plus),
                    '-' => LexerToken::Punctuation(Punctuation::Minus),
                    '*' => LexerToken::Punctuation(Punctuation::Mul),
                    '/' => LexerToken::Punctuation(Punctuation::Div),

                    '>' if it.peek() == Some(&'=') => {
                        it.next();
                        LexerToken::Punctuation(Punctuation::GE)
                    }
                    '<' if it.peek() == Some(&'=') => {
                        it.next();
                        LexerToken::Punctuation(Punctuation::LE)
                    }
                    '=' if it.peek() == Some(&'=') => {
                        it.next();
                        LexerToken::Punctuation(Punctuation::Equals)
                    }

                    '<' => LexerToken::Punctuation(Punctuation::LT),
                    '>' => LexerToken::Punctuation(Punctuation::GT),
                    '=' => LexerToken::Punctuation(Punctuation::Assign),

                    _ => {
                        return Err(ParseError {
                            msg: format!("Invalid punctuation: {}", c),
                        });
                    }
                };

                ret.push(tok);
            }

            _ => {
                let msg = format!("Unknow char: '{}'", c);
                return Err(ParseError { msg });
            }
        }
    }

    Ok(ret)
}
