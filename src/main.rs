mod parser;
mod lexer;

use std::env;
use std::fs::File;
use parser::ParserData;




fn main() {

    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    if args.len() <= 1 {
        println!("File name not specify");
        return;
    }

    println!("File name: {}", file_path);

    let file_content = File::open(file_path)
        .expect("File not exist or cannot be opened");


    let data = ParserData::from_file(file_content).unwrap();
    let (mut line, mut address) = (1, data.header.start_address);


    println!("{:>04} {:>08} {:>12} {:>12} {:>12} {}", "line", "address", "label", "operate", "operand", "opcode");

    for text in &data.texts {
        for (expression, opcode) in &text.expressions {

            println!("{:>04} {:>8X} {} {:>}", line, address, expression, opcode);

            line += 1;
            address += expression.len() as u64;
        }
    }

    println!("{}", data.header);

    for text in &data.texts {
        println!("{}", text);
    }

    println!{"{}", data.end};
}
