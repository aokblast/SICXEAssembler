mod token;
pub mod expression;
pub mod command;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use expression::*;
use command::*;
use token::*;


// TODO: Check RSUB
pub struct HeaderSection {
    pub program_name: String,
    pub start_address: u64,
    pub len: u64
}

impl Display for HeaderSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "H{:6}{:06X}{:06X}", self.program_name, self.start_address, self.len)
    }
}

impl HeaderSection {
    pub fn from_expression(expression: &Expression) -> Result<Self, &str> {
        if let (Command::Directive(Directive::START), _) = expression.command {
            if let Some((Operand::Literal(Literal::Integer(start_addr)), _)) = expression.operand {
                Ok(Self{program_name: String::from(expression.label.as_ref().unwrap()), start_address: (u64::from_str_radix(&(start_addr).to_string(), 16) as u64), len: 0})
            } else {
                Err("Invalid Literal")
            }
        } else {
            Err("Not start directive")
        }
    }
}

pub struct TextSection {
    pub expressions: Vec<(Expression, String)>,
    pub start_address: u64,
    pub len: u64
}

impl Display for TextSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for (_, opcode) in &self.expressions {
            str += opcode;
        }
        write!(f, "T{:<06X}{:<02X}{}", self.start_address, self.len, str)
    }
}

pub struct EndSection {
    start_address: u64
}

impl Display for EndSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{:06X}", self.start_address)
    }
}

impl EndSection {
    pub fn from_expression(expression: &Expression, symbol_table: &HashMap<String, u64>) -> Result<Self, Box<dyn Error>> {
        if let (Command::Directive(Directive::END), _) = expression.command {
            if let Some((Operand::Symbol(symbol), _)) = &expression.operand {
                if let Some(addr) = symbol_table.get(symbol){
                    Ok(Self{start_address: *addr})
                } else {
                    Err(format!("Symbol: {} not found", symbol))?
                }

            } else {
                Err("Invalid Symbol in end text")?
            }
        } else {
            Err("Not end directive")?
        }
    }
}


pub struct ParserData {
    pub symbol_table: HashMap<String, u64>,
    pub header: HeaderSection,
    pub texts: Vec<TextSection>,
    pub end: EndSection
}

impl ParserData {

    fn get_addr(addr: u64, pc: u64, base: &Option<u64>) -> (i64, Option<Flag>){
        let addr = addr as i64;
        let pc = pc as i64;

        if (addr - pc) >= -2048 && (addr - pc) <= 2047  {
            (addr - pc, Some(Flag::P))
        } else if let Some(base) = *base {
            let base = base as i64;
            if (addr - base) >= 0 && (addr - base) <= 4095 {
                (addr - base, Some(Flag::B))
            } else {
                (addr, None)
            }

        } else {
            (addr, None)
        }
    }

    fn get_symbols(expressions: &Vec<Expression>, start_address: u64) -> Result<HashMap<String, u64>, &str> {
        let mut symbol_table = HashMap::new();
        let mut addr = start_address;

        for expression in expressions {
            if let Some(label) = &expression.label {
                match symbol_table.get(label) {
                    Some(_) => {
                        return Err("Duplicated Symbol");
                    }
                    None => {
                        symbol_table.insert(String::from(label), addr);
                    }
                }
            }
            addr += expression.len() as u64;
        }

        Ok(symbol_table)
    }

    fn parse(expressions: &Vec<Expression>, symbol_table: &HashMap<String, u64>, start_address: u64) -> Result<Vec<String>, Box<dyn Error>>{
        let mut res = vec![];
        let mut base = None;
        let mut pc = start_address;

        for expression in &expressions[1..expressions.len() - 1] {
            pc += expression.len() as u64;

            match &expression.command.0 {
                Command::Directive(directive) => {
                    match directive {
                        Directive::RESB => {
                            res.push("".to_string());
                        }
                        Directive::RESW => {
                            res.push("".to_string());
                        }
                        Directive::BYTE => {
                            match &expression.operand.as_ref().unwrap().0 {
                                Operand::Literal(literal) => {
                                    match literal {
                                        Literal::Integer(num) => {
                                            res.push(format!("{:02X}", num));
                                        }
                                        Literal::String(str) => {
                                            res.push(hex::encode_upper(str));
                                        }
                                        _ => {
                                            Err("Invalid expression")?
                                        }
                                    }
                                }
                                Operand::Symbol(_) => {
                                    return Err("Invalid expression")?
                                }
                            }
                        }
                        Directive::WORD => {
                            match &expression.operand.as_ref().unwrap().0 {
                                Operand::Literal(literal) => {
                                    match literal {
                                        Literal::Integer(num) => {
                                            res.push(format!("{:04X}", num));
                                        }
                                        _ => {
                                            Err("Invalid expression")?
                                        }
                                    }
                                }
                                Operand::Symbol(_) => {
                                    Err("Invalid expression")?
                                }
                            }
                        }
                        Directive::BASE => {
                            match &expression.operand.as_ref().unwrap().0 {
                                Operand::Symbol(symbol) => {
                                    if let Some(addr) = symbol_table.get(symbol) {
                                        base = Some(*addr);
                                        res.push("".to_string());
                                    } else {
                                        Err(format!("Symbol {} not found", symbol))?
                                    }
                                }
                                Operand::Literal(_) => {
                                    Err("Invalid expression")?
                                }
                            }
                        }
                        _ => {
                            Err("Start and End cannot be in the text segment")?
                        }
                    }
                }
                Command::Mnemonic(mnemonic) => {
                    let mut code: u32 = 0;
                    let opcode = mnemonic.opcode as u32;
                    match mnemonic.format {
                        Format::ONE => {
                            code |= opcode << 16;
                            res.push(format!("{: <06X}", code));
                        }
                        Format::TWO => {
                            code |= opcode << 8;
                            if let Operand::Literal(Literal::RegisterPair((r1, r2))) = expression.operand.as_ref().unwrap().0 {
                                code |= ((r1 as u32) << 4) | (r2 as u32);
                                res.push(format!("{: <04X}", code));
                            } else {
                                Err("Invalid expression")?
                            }
                        }
                        Format::ThreeAndFour => {
                            if expression.stat.is_set(Flag::E) {
                                code |= (opcode & 0xFC) << 24;
                            } else {
                                code |= (opcode & 0xFC) << 16;
                            }
                            let mut stat = expression.stat;
                            let mut addr = 0u32;
                            match &expression.operand.as_ref().unwrap().0 {
                                Operand::Symbol(symbol) => {
                                    if let Some(address) = symbol_table.get(symbol) {
                                        if expression.stat.is_set(Flag::E) {
                                            addr = (((*address) as u32) & ((1u32 << 20) - 1u32)) as u32;
                                        } else {
                                            let (bias, flag) = Self::get_addr(*address, pc, &base);
                                            if let Some(flag) = flag {
                                                stat.set(flag);
                                                addr = (bias as u32) & ((1u32 << 12) - 1) as u32;
                                            } else {
                                                addr = (bias as u32) & ((1u32 << 12) - 1) as u32;
                                            }
                                        }
                                    } else {
                                        Err(format!("Symbol {} not found", symbol))?
                                    }
                                }
                                Operand::Literal(literal) => {
                                    match literal {
                                        Literal::Integer(num) => {
                                            addr = *num as u32;
                                        }
                                        _ => {
                                            Err("Invalid expression")?
                                        }
                                    }
                                }
                            }
                            if expression.stat.is_set(Flag::E) {
                                code |= (stat.get_val() as u32) << 20;
                                code |= addr;
                                res.push(format!("{: <08X}", code));
                            } else {
                                code |= (stat.get_val() as u32) << 12;
                                code |= addr;
                                res.push(format!("{: <06X}", code));
                            }

                        }
                    }

                }
            }
        }
        Ok(res)
    }

    pub fn from_file(file: File) -> Result<Self, &'static str> {
        let mut expressions = vec![];
        let mut line_cnt = 1;

        let lines = BufReader::new(file).lines();

        for line in lines {
            if let Ok(line) = line {
                let expression = Expression::from_str(&line)
                    .expect(format!("Incorrect instruction on line {}", line_cnt).as_str());
                expressions.push(expression);
                line_cnt += 1;
            }
        }

        let mut header = HeaderSection::from_expression(&expressions[0]).unwrap();
        let symbol_table = Self::get_symbols(&expressions, header.start_address).unwrap();
        let op_codes = Self::parse(&expressions, &symbol_table, header.start_address).unwrap();
        let end = EndSection::from_expression(&expressions[expressions.len() - 1], &symbol_table).unwrap();
        let mut texts =  vec![];
        let mut cur_text: TextSection = TextSection{expressions: vec![], start_address: header.start_address, len: 0};
        let mut cur_addr = header.start_address;
        let mut has_no_resv = false;

        for idx in 1..(expressions.len() - 1) {
            if cur_addr - cur_text.start_address + (expressions[idx].len() as u64) > 0x1D && has_no_resv {
                cur_text.len = cur_addr - cur_text.start_address;
                texts.push(cur_text);
                cur_text = TextSection{expressions: vec![], start_address: cur_addr, len: 0};
                has_no_resv = false;
            }

            match expressions[idx].command.0 {
                Command::Directive(Directive::RESW) | Command::Directive(Directive::RESB) => {
                    if has_no_resv {
                        cur_text.len = cur_addr - cur_text.start_address;
                        texts.push(cur_text);
                        cur_text = TextSection{expressions: vec![], start_address: cur_addr, len: 0};
                        has_no_resv = false;
                    }
                }
                _ => {
                    if !has_no_resv {
                        cur_text.start_address = cur_addr;
                    }
                    has_no_resv = true;
                }

            }

            cur_text.expressions.push((expressions[idx].clone(), op_codes[idx - 1].clone()));
            cur_addr += expressions[idx].len() as u64;
        }

        if cur_text.expressions.len() != 0 {
            cur_text.len = cur_addr - cur_text.start_address;
            texts.push(cur_text);
        }

        header.len = cur_addr;
        Ok(Self{symbol_table, header, texts, end})
    }
}