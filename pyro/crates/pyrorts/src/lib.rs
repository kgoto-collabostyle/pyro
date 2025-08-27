use pyroc::{BinOp, Expr, Module, Stmt};

pub fn generate(m: &Module) -> String {
    let mut body = String::new();
    for stmt in &m.stmts {
        body.push_str(&generate_stmt(stmt));
    }
    format!("fn main() {{\n{}\n}}\n", indent(&body, 4))
}

// ステートメント単位のコード生成（mainはここでは作らない）
fn generate_stmt(stmt: &Stmt) -> String {
    let mut code = String::new();
    match stmt {
        // print(...) を特別扱いして出力
        Stmt::Expr(Expr::Call { callee, args }) if callee == "print" => {
            if let Some(arg) = args.get(0) {
                code.push_str(&format!("println!(\"{{}}\", {});\n", expr_to_rust(arg)));
            } else {
                code.push_str("println!(\"<print: missing arg>\");\n");
            }
        }

        // それ以外の式文は現状 no-op
        Stmt::Expr(_) => { /* no-op */ }

        // 変数宣言 letpr
        Stmt::Let { name, expr } => {
            code.push_str(&format!("let {} = {};\n", name, expr_to_rust(expr)));
        }

        // if 文（数値として 0 以外を真とみなす）
        Stmt::If {
            cond,
            then_block,
            else_block,
        } => {
            code.push_str(&format!("if ({} as f64) != 0.0 {{\n", expr_to_rust(cond)));
            for s in then_block {
                code.push_str(&indent(&generate_stmt(s), 4));
            }
            code.push_str("}\n");
            if !else_block.is_empty() {
                code.push_str("else {\n");
                for s in else_block {
                    code.push_str(&indent(&generate_stmt(s), 4));
                }
                code.push_str("}\n");
            }
        }
    }
    code
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
        // ここは未使用（print は Stmt 側で処理するため）
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
