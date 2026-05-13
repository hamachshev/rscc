use clap::Parser;
use rscc::codegen::CodeGen;
use rscc::{
    lexer::{self, Lexer},
    parser,
};
use std::{
    ffi::OsString,
    fs::DirBuilder,
    io::{self, Read, Write},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

#[allow(unused)]
fn main() {
    let args = Args::parse();
    let mut path: PathBuf = args.file;
    let output_filename = args.output_filename;
    let output_to_std_out = args.output_to_sdout == Some('-');
    let output_ast = args.output_ast.unwrap_or(false);
    let output_tokens = args.output_tokens.unwrap_or(false);

    let mut file = Vec::new();
    std::fs::File::open(&path).unwrap().read_to_end(&mut file);

    let lex = Lexer::new().lex(&file[..]).unwrap();

    if output_tokens {
        println!("{:#?}", lex);
        return;
    }
    let parse = parser::Parser::new(&file[..]).parse_program(&mut lex.into_iter().peekable());

    if output_ast {
        println!("{:#?}", parse);
        return;
    }
    let codegen = CodeGen::new().gen_program(parse);

    let output_filename = match output_filename {
        Some(n) => OsString::from(n),
        None => {
            let c_path = path;
            let mut path = PathBuf::from("build/");
            if !path.exists() {
                DirBuilder::new()
                    .create(&path)
                    .expect("failed to create build dir")
            }
            path.push(c_path);
            path.set_extension("s");
            OsString::from(path)
        }
    };
    let output_filename_string = String::from_utf8_lossy(output_filename.as_bytes());
    if output_to_std_out {
        io::stdout()
            .write_all(codegen.as_bytes())
            .expect("couldnt write to stdout");
    } else {
        std::fs::write(&output_filename, codegen).expect(&format!(
            "couldnt write to file {}",
            &output_filename_string
        ));
        println!("compiled to {}", output_filename_string);
    }
}

#[cfg(test)]
mod test {
    use std::{fs, process::Command};

    #[test]
    fn test() {
        let valid_tests = fs::read_dir("write_a_c_compiler-master/stage_2/valid").unwrap();
        let invalid_tests = fs::read_dir("write_a_c_compiler-master/stage_2/invalid").unwrap();

        for test in valid_tests.chain(invalid_tests) {
            let test = test.unwrap();
            println!("{}", test.path().to_str().unwrap());
            let res = Command::new("./target/debug/rscc")
                .arg(test.path().to_str().unwrap())
                .output()
                .unwrap();
            println!("{}", String::from_utf8_lossy(&res.stdout));
            println!("{}", &res.status);
        }
    }
}

///rscc - c compiler written in rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: PathBuf,
    /// output file path. default is <path-of-c-file>.s
    #[arg(short)]
    output_filename: Option<PathBuf>,

    ///output ast
    #[arg(short = 'a')]
    output_ast: Option<bool>,

    ///output tokens
    #[arg(short = 't')]
    output_tokens: Option<bool>,

    /// Output to stdout instead of file
    output_to_sdout: Option<char>,
}
