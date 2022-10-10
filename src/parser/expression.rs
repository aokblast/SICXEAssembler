use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::lexer;
use crate::parser::command::*;
use crate::parser::token::{Token, Literal};

#[derive(Clone)]
pub enum Operand {
    Literal(Literal),
    Symbol(String),
}

#[derive(Clone)]
pub struct Expression {
    pub command: (Command, String),
    pub operand: Option<(Operand, String)>,
    pub label: Option<String>,
    pub stat: Stat
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let label : &str;
        let operand : &str;
        let default = "";

        if let Some(lab) = &self.label {
            label = lab;
        } else {
            label = default;
        }

        if let Some(op) = &self.operand {
            operand = &op.1;
        } else {
            operand = default;
        }

        write!(f, "{: >12} {: >12} {: >12}", label, &self.command.1, operand)
    }
}

impl Expression {

    fn is_valid(self: Self) -> Result<Self, Box<dyn Error>> {
        // TODO: check size of operand
        // TODO: check type 2 operand
        match self.command.0 {
            Command::Directive(d) => {
                match d {
                    Directive::START | Directive::BYTE | Directive::WORD | Directive::RESB | Directive::RESW => {
                        if self.label.is_some() && self.operand.is_some() {
                            if d == Directive::BYTE {
                                if let Operand::Literal(Literal::RegisterPair(_)) = &(self.operand.as_ref().unwrap().0) {
                                    Err("Operand must not be Regist file for BYTE directive")?
                                } else{
                                    Ok(self)
                                }
                            } else if let Operand::Literal(Literal::Integer(_)) = &(self.operand.as_ref().unwrap().0) {
                                Ok(self)
                            } else {
                                Err("Operand must be Integer for directive")?
                            }
                        } else {
                            Err("Miss label or operand for directive")?
                        }
                    }

                    Directive::END | Directive::BASE => {
                        if self.label.is_none() {
                            Ok(self)
                        } else {
                            Err("Must with no label for END directive")?
                        }
                    }
                }
            }

            Command::Mnemonic(m) => {
                match m.format {
                    Format::ONE => {
                        if self.stat.get_val() == 0 && self.operand.is_none() {
                            Ok(self)
                        } else {
                            Err("Error format for format 1 instruction")?
                        }
                    }
                    Format::TWO => {
                        if self.stat.get_val() == 0 && self.operand.is_some() {
                            if let Operand::Literal(Literal::RegisterPair(_)) = &(self.operand.as_ref().unwrap().0) {
                                Ok(self)
                            } else {
                                Err("Must be Register Pair operand for format 2 instruction")?
                            }
                        } else {
                            Err("Error format for format 2 instruction")?
                        }
                    }
                    Format::ThreeAndFour => {
                        if self.operand.is_some() {
                            match &(self.operand.as_ref().unwrap().0) {
                                Operand::Literal(lit) => {
                                    if let Literal::RegisterPair(_) = lit {
                                        Err("Must not be Register Pair operand for format 3/4 instruction")?
                                    } else {
                                        Ok(self)
                                    }
                                }
                                Operand::Symbol(_) => {
                                    Ok(self)
                                }

                            }
                        } else {
                            Err("Miss operand for format 3/4 instruction")?
                        }
                    }
                }
            }
        }
    }

    pub fn len(self: &Self) -> usize {
        match self.command.0 {
            Command::Mnemonic(mnemonic) => {
                mnemonic.format.len() + (self.stat.is_set(Flag::E) as usize)
            }
            Command::Directive(directive) => {
                match directive {
                    Directive::RESW | Directive::RESB => {
                        if let Some((Operand::Literal(Literal::Integer(size)), _)) = self.operand {
                            directive.len() * (size as usize)
                        } else {
                            directive.len()
                        }
                    }
                    Directive::BYTE => {
                        if let Some((Operand::Literal(Literal::String(str)), _)) = &self.operand {
                            directive.len() * str.len()
                        } else {
                            directive.len()
                        }
                    }

                    _ => {
                        directive.len()
                    }
                }
            }
        }
    }

    pub fn from_str(line: &str) -> Result<Expression, Box<dyn Error>> {
        let lexemas = lexer::parse_line_to_lexemas(line);
        let tokens = Token::from_lexemas(&lexemas);

        let mut command = None;
        let mut label = None;
        let mut operand = None;

        if let Err(err) = tokens {
            return Err(err);
        }

        let (tokens, stat) = Token::from_lexemas(&lexemas).unwrap();

        match tokens.len() {
            1 => {
                match &tokens[0] {
                    Token::Command(cmd) => {
                        command = Some((*cmd, lexemas[0].clone()));
                    }
                    _ => {
                        Err("Invalid expression(There must be a instruction in one line)")?
                    }
                }
            }

            2 => {
                match &tokens[0] {
                    Token::Symbol(sym) => {
                        label = Some(String::from(sym));
                    }
                    Token::Command(cmd) => {
                        command = Some((*cmd, lexemas[0].clone()));
                    }
                    _ => {
                        Err("Invalid expression(First Argument must be a Symbol)")?
                    }
                }

                match &tokens[1] {
                    Token::Command(cmd) => {
                        match command {
                            None => {
                                command = Some((*cmd, lexemas[1].clone()));
                            }
                            Some(_) => {
                                Err("Invalid expression(Too many command in one line)")?
                            }

                        }
                    }
                    Token::Symbol(sym) => {
                        operand = Some((Operand::Symbol(String::from(sym)), lexemas[1].clone()));
                    }
                    Token::Literal(lit) => {
                        operand = Some((Operand::Literal(lit.clone()), lexemas[1].clone()));
                    }
                }
            }

            3 => {
                match &tokens[0] {
                    Token::Symbol(sym) => {
                        label = Some(String::from(sym));
                    }
                    _ => {
                        Err("Invalid expression(first token must be symbol)")?
                    }
                }
                match &tokens[1] {
                    Token::Command(cmd) => {
                        command = Some((*cmd, lexemas[1].clone()));
                    }
                    _ => {
                        Err("Invalid expression(second token must be instruction)")?
                    }
                }
                match &tokens[2] {
                    Token ::Literal(lit) => {
                        operand = Some((Operand::Literal(lit.clone()), lexemas[2].clone()));
                    }
                    Token::Symbol(sym) => {
                        operand = Some((Operand::Symbol(String::from(sym)), lexemas[2].clone()));
                    }
                    _ => {
                        Err("Invalid expression(third token must not be instruction)")?
                    }
                }
            }

            _ => {
                Err("Invalid expression:(Too many token in one expression)")?
            }
        }

        if let None = command {
            Err("Invalid expression: Instruction not found")?
        }

        let mut res = Self{command: command.unwrap(), operand, label, stat };

        if let (Command::Mnemonic(Mnemonic{opcode: _opcode, format: Format::ThreeAndFour}), _) = res.command {
            if !stat.is_set(Flag::N) && !stat.is_set(Flag::I) {
                res.stat.set(Flag::N);
                res.stat.set(Flag::I);
            }
        }

        res.is_valid()
    }
}



