use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use pyroc::parser::Parser as PyroParser;
use pyrorts::generate;

#[derive(Parser)]
#[command(name = "pyro")]
#[command(about = "Pyro language transpiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run,
    Build,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run => {
            let src = fs::read_to_string("examples/main.pyro").expect("failed to read source");
            let mut parser = PyroParser::new(&src).expect("lexer failed");
            let m = parser.parse_module().expect("parse failed");
            let out = generate(&m);

            // ✅ 出力ディレクトリ
            let out_dir = PathBuf::from("pyro/output");
            fs::create_dir_all(&out_dir).expect("failed to create output dir");

            // ✅ out.rs は pyro/output に保存
            let out_rs = out_dir.join("out.rs");
            fs::write(&out_rs, out).expect("write out.rs failed");

            // ✅ 実行ファイルは pyro/output/bin/out に保存
            let bin_dir = out_dir.join("bin");
            fs::create_dir_all(&bin_dir).expect("failed to create bin dir");
            let out_bin = bin_dir.join("out");

            println!("✅ Transpiled to {}", out_rs.display());
            println!("✅ Compiled binary -> {}", out_bin.display());

            // コンパイル
            let status = Command::new("rustc")
                .arg(&out_rs)
                .arg("-o")
                .arg(&out_bin)
                .status()
                .expect("failed to run rustc");
            assert!(status.success());

            // 実行
            Command::new(&out_bin)
                .status()
                .expect("failed to run program");
        }
        Commands::Build => {
            let status = Command::new("cargo")
                .args(&["clean"])
                .status()
                .expect("failed to run cargo clean");
            assert!(status.success());
            let status = Command::new("cargo")
                .args(&["build"])
                .status()
                .expect("failed to run cargo build");
            assert!(status.success());
        }
    }
}
