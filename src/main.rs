use crate::parser::Parser;
use interpreter::Interpreter;
// use pprof::protos::Message;
use scanner::Scanner;
use std::fs;
// use std::fs::File;
// use std::io::Write;

mod ast;
mod error;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod types;

fn main() {
    // let guard = pprof::ProfilerGuard::new(997).unwrap();

    let code = fs::read_to_string("test.wind").unwrap();

    let mut scanner = Scanner::new(code);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(ast);

    // if let Ok(report) = guard.report().build() {
    //     let mut file = File::create("profile.pb").unwrap();
    //     let profile = report.pprof().unwrap();

    //     let mut content = Vec::new();
    //     profile.encode(&mut content).unwrap();
    //     file.write_all(&content).unwrap();
    // };
}
