use crate::{BinOp, Expr, Module, Stmt};

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Str(String),
    Num(f64),
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Newline,
    Eof,
    Eq,
}

struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    peeked: Option<char>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars(),
            peeked: None,
        }
    }

    fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }
        self.peeked
    }
    fn bump(&mut self) -> Option<char> {
        if let Some(c) = self.peeked.take() {
            Some(c)
        } else {
            self.chars.next()
        }
    }

    fn next_token(&mut self) -> Result<Tok, String> {
        // 空白/タブ/CR をスキップ、# 以降は改行までコメントとして捨てる
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.bump();
                }
                Some('#') => {
                    // コメント本体を読み飛ばす（行末まで）
                    while let Some(c) = self.bump() {
                        if c == '\n' {
                            break;
                        }
                    }
                    // コメント行は「改行」として扱う
                    return Ok(Tok::Newline);
                }
                _ => break,
            }
        }

        match self.bump() {
            None => Ok(Tok::Eof),
            Some('\n') => Ok(Tok::Newline),
            Some('(') => Ok(Tok::LParen),
            Some(')') => Ok(Tok::RParen),
            Some(',') => Ok(Tok::Comma),
            Some('+') => Ok(Tok::Plus),
            Some('-') => Ok(Tok::Minus),
            Some('*') => Ok(Tok::Star),
            Some('/') => Ok(Tok::Slash),
            Some('=') => Ok(Tok::Eq),
            Some('"') => self.lex_string(),
            Some(c) if is_ident_start(c) => {
                let mut s = String::new();
                s.push(c);
                while let Some(nc) = self.peek() {
                    if is_ident_cont(nc) {
                        s.push(nc);
                        self.bump();
                    } else {
                        break;
                    }
                }
                Ok(Tok::Ident(s))
            }
            Some(c) if c.is_ascii_digit() => {
                let mut s = String::new();
                s.push(c);
                while let Some(nc) = self.peek() {
                    if nc.is_ascii_digit() || nc == '.' {
                        s.push(nc);
                        self.bump();
                    } else {
                        break;
                    }
                }
                let v: f64 = s
                    .parse()
                    .map_err(|_| format!("invalid number literal: {}", s))?;
                Ok(Tok::Num(v))
            }
            Some(c) => Err(format!("unexpected char: {}", c)),
        }
    }

    fn lex_string(&mut self) -> Result<Tok, String> {
        let mut s = String::new();
        while let Some(c) = self.bump() {
            match c {
                '\\' => match self.bump() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some(other) => s.push(other),
                    None => return Err("unterminated escape".into()),
                },
                '"' => return Ok(Tok::Str(s)),
                _ => s.push(c),
            }
        }
        Err("unterminated string".into())
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
fn is_ident_cont(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

pub struct Parser<'a> {
    toks: Vec<Tok>,
    i: usize,
    _src: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Result<Self, String> {
        let mut lx = Lexer::new(src);
        let mut toks = Vec::new();
        loop {
            let t = lx.next_token()?;
            let is_eof = matches!(t, Tok::Eof);
            toks.push(t);
            if is_eof {
                break;
            }
        }
        Ok(Self {
            toks,
            i: 0,
            _src: src,
        })
    }

    fn at(&self) -> &Tok {
        self.toks.get(self.i).unwrap_or(&Tok::Eof)
    }
    fn bump(&mut self) {
        if self.i < self.toks.len() {
            self.i += 1;
        }
    }
    fn eat_newlines(&mut self) {
        while matches!(self.at(), Tok::Newline) {
            self.bump();
        }
    }

    pub fn parse_module(&mut self) -> Result<Module, String> {
        let mut stmts = Vec::new();
        self.eat_newlines();
        while !matches!(self.at(), Tok::Eof) {
            if matches!(self.at(), Tok::Newline) {
                self.bump();
                continue;
            }
            // letpr 文の処理
            if let Tok::Ident(ref s) = self.at() {
                if s == "letpr" {
                    self.bump(); // 'letpr'
                    let name = if let Tok::Ident(n) = self.at().clone() {
                        self.bump();
                        n
                    } else {
                        return Err("expected identifier after letpr".into());
                    };
                    if !matches!(self.at(), Tok::Eq) {
                        return Err("expected '=' after identifier".into());
                    }
                    self.bump(); // '='
                    let expr = self.parse_expr(0)?;
                    stmts.push(Stmt::Let { name, expr });
                    self.eat_newlines();
                    continue;
                }
            }

            let e = self.parse_expr(0)?;
            stmts.push(Stmt::Expr(e));
            self.eat_newlines();
        }
        Ok(Module { stmts })
    }

    // 優先順位: * / (20) > + - (10)（左結合）
    fn precedence(op: &Tok) -> Option<u8> {
        match op {
            Tok::Plus | Tok::Minus => Some(10),
            Tok::Star | Tok::Slash => Some(20),
            _ => None,
        }
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_primary()?;
        loop {
            let op_tok = self.at().clone();
            let prec = if let Some(p) = Self::precedence(&op_tok) {
                p
            } else {
                break;
            };
            if prec < min_bp {
                break;
            }
            self.bump(); // consume op
            let rhs = self.parse_expr(prec + 1)?; // left assoc
            lhs = match op_tok {
                Tok::Plus => Expr::Binary {
                    op: BinOp::Add,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                Tok::Minus => Expr::Binary {
                    op: BinOp::Sub,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                Tok::Star => Expr::Binary {
                    op: BinOp::Mul,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                Tok::Slash => Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                _ => unreachable!(),
            };
        }
        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.at().clone() {
            Tok::Num(v) => {
                self.bump();
                Ok(Expr::Number(v))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Expr::Str(s))
            }
            Tok::Ident(name) => {
                self.bump();
                if matches!(self.at(), Tok::LParen) {
                    // 関数呼び出し（print 等）
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.at(), Tok::RParen) {
                        loop {
                            let arg = self.parse_expr(0)?;
                            args.push(arg);
                            match self.at() {
                                Tok::Comma => {
                                    self.bump();
                                }
                                Tok::RParen => break,
                                t => return Err(format!("expected ',' or ')', got {:?}", t)),
                            }
                        }
                    }
                    if !matches!(self.at(), Tok::RParen) {
                        return Err("expected ')'".into());
                    }
                    self.bump();
                    Ok(Expr::Call { callee: name, args })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Tok::LParen => {
                self.bump();
                let e = self.parse_expr(0)?;
                if !matches!(self.at(), Tok::RParen) {
                    return Err("expected ')'".into());
                }
                self.bump();
                Ok(e)
            }
            other => Err(format!("unexpected token: {:?}", other)),
        }
    }
}
