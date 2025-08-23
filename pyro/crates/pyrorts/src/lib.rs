use pyroc::{BinOp, Expr, Module, Stmt};

pub fn generate(m: &Module) -> String {
    let mut body = String::new();

    for stmt in &m.stmts {
        match *stmt {
            // Pyro独自: 変数宣言 `letpr`
            Stmt::Let { ref name, ref expr } => {
                body.push_str(&format!("let {} = {};\n", name, expr_to_rust(expr)));
            }

            // 組み込み: print(expr)
            Stmt::Expr(Expr::Call {
                ref callee,
                ref args,
            }) if callee == "print" => {
                if let Some(arg) = args.get(0) {
                    let expr_rs = expr_to_rust(arg);
                    body.push_str(&format!("println!(\"{{}}\", {});\n", expr_rs));
                } else {
                    body.push_str("println!(\"<print: missing arg>\");\n");
                }
            }

            // それ以外の式文は現状は無視（必要なら評価コードを生成する）
            Stmt::Expr(_) => { /* no-op */ }
        }
    }

    format!("fn main(){{\n{}\n}}\n", indent(&body, 4))
}

fn expr_to_rust(e: &Expr) -> String {
    match e {
        Expr::Number(n) => n.to_string(),
        Expr::Str(s) => format!("\"{}\"", escape_rust(s)),
        Expr::Ident(name) => name.clone(),
        Expr::Binary { op, lhs, rhs } => {
            let l = expr_to_rust(lhs);
            let r = expr_to_rust(rhs);
            let op_str = match *op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
            };
            format!("({} {} {})", l, op_str, r)
        }
        Expr::Call { .. } => "\"<unsupported call>\"".to_string(),
    }
}

fn indent(s: &str, n: usize) -> String {
    let pad = " ".repeat(n);
    s.lines()
        .map(|ln| format!("{}{}", pad, ln))
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_rust(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}
