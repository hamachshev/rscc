use std::{
    env::args,
    io::{self, Bytes, Read},
    iter::Peekable,
    path::PathBuf,
};

#[allow(unused)]
fn main() {
    let path: PathBuf = args()
        .nth(1)
        .expect("Must input a c file to compile")
        .into();

    let file = std::fs::File::open(path).unwrap();
    println!("{:?}", lex(file))
}

#[allow(unused)]
fn lex(source: impl Read) -> Result<Vec<Token>, io::Error> {
    let mut tokens = Vec::new();
    let mut iterator = source.bytes().peekable();

    while let Some(Ok(byte)) = iterator.next() {
        if byte == b' ' || byte == b'\t' || byte == b'\n' {
            continue;
        }
        let token = match byte {
            b'{' => Token::CurlyBraceOpen,
            b'}' => Token::CurlyBraceClose,
            b'(' => Token::ParenOpen,
            b')' => Token::ParenClose,
            b';' => Token::SemiColon,
            n if is_num(n) => lex_num(n, &mut iterator),
            a if is_alpha(a) => lex_alpha(a, &mut iterator),
            u => Token::Unknown(String::from_utf8_lossy(&[u]).to_string()),
        };
        tokens.push(token);
    }
    Ok(tokens)
}

fn is_num(byte: u8) -> bool {
    b'0' <= byte && byte <= b'9'
}
fn lex_num(num_start: u8, iterator: &mut Peekable<Bytes<impl Read>>) -> Token {
    let mut num = vec![num_start];
    while let Some(Ok(byte)) = iterator.peek() {
        if is_num(*byte) {
            num.push(iterator.next().unwrap().unwrap());
        } else {
            break;
        }
    }
    Token::Integer(String::from_utf8(num).unwrap().parse::<usize>().unwrap())
}
fn is_alpha(byte: u8) -> bool {
    b'a' <= byte && byte <= b'z' || b'A' <= byte && byte <= b'Z'
}
fn lex_alpha(str_start: u8, iterator: &mut Peekable<Bytes<impl Read>>) -> Token {
    let mut string = vec![str_start];
    while let Some(Ok(byte)) = iterator.peek() {
        if is_alpha(*byte) {
            string.push(iterator.next().unwrap().unwrap());
        } else {
            break;
        }
    }
    let string = String::from_utf8(string).unwrap();
    match string.as_ref() {
        "return" => Token::Return,
        "int" => Token::Int,
        _ => Token::Ident(string),
    }
}
#[derive(Debug)]
enum Token {
    CurlyBraceOpen,
    CurlyBraceClose,
    ParenOpen,
    ParenClose,
    SemiColon,
    Return,
    Int,
    Ident(String),
    Integer(usize),
    Unknown(String),
}

#[cfg(test)]
mod test {
    use std::{fs, process::Command};

    #[test]
    fn test() {
        let valid_tests = fs::read_dir("write_a_c_compiler-master/stage_1/valid").unwrap();
        let invalid_tests = fs::read_dir("write_a_c_compiler-master/stage_1/invalid").unwrap();

        for test in valid_tests.chain(invalid_tests) {
            let test = test.unwrap();
            println!("{}", test.path().to_str().unwrap());
            let res = Command::new("./target/debug/rscc")
                .arg(test.path().to_str().unwrap())
                .output()
                .unwrap();
            println!("{}", String::from_utf8_lossy(&res.stdout))
        }
    }
}
