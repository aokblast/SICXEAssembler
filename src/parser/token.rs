use std::error::Error;
use std::str::FromStr;
use crate::parser::command::*;

#[derive(Clone)]
pub enum Literal{
    String(String),
    Integer(i32),
    RegisterPair((Register, Register))
}

impl Literal {
    fn has_string(s: &str) -> bool {
        s.len() > 3 && s.chars().nth(1).unwrap() == '\'' && s.chars().nth(s.len() - 1).unwrap() == '\''
    }

    fn get_register_pair(s: &str) -> Option<(Register, Register)> {
        if s.len() == 3 && s.chars().nth(1).unwrap() == ',' {
            Some((Register::from_char(s.chars().nth(0).unwrap()).expect("Register not found"),
                  Register::from_char(s.chars().nth(2).unwrap()).expect("Register not found"))
                )
        } else if s.len() == 1 && s.chars().nth(0).unwrap().is_alphabetic() {
            Some((Register::from_char(s.chars().nth(0).unwrap()).expect("Register not found"),
                  Register::from_char('A').expect("Register not found"))
                )
        } else {
            None
        }
    }
}

impl FromStr for Literal {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if Self::has_string(s) {
            match s.chars().nth(0).unwrap() {
                'X' => {
                    Ok(Self::Integer(i32::from_str_radix(&s[2..s.len() - 1], 16).expect("Not a number")))
                }
                'C' => {
                    Ok(Self::String(String::from(&s[2..s.len() - 1])))
                }
                _ => {
                    Err("Invalid literal with parenthesis")?
                }
            }
        } else {
            if let Ok(num) = s.parse::<i32>() {
                Ok(Self::Integer(num))
            } else if let Some(pair) = Self::get_register_pair(s){
                Ok(Self::RegisterPair(pair))
            } else {
                Err("Invalid literal")?
            }
        }

    }

}

pub enum Token {
    Literal(Literal),
    Symbol(String),
    Command(Command),
}


impl Token {
    pub fn is_symbol(lexema: &str) -> bool {
        if lexema.len() == 1 {
            return false;
        }
        let prefix = lexema.chars().nth(0).unwrap();

        if !prefix.is_alphabetic() && prefix != '_' {
            false
        } else if lexema.contains(|x: char| {!x.is_alphanumeric()}) {
            false
        } else {
            true
        }
    }

    fn is_valid(self: &Self, prefix: &Option<Flag>, suffix: &Option<Flag>) -> bool {
        if prefix.is_some() && suffix.is_some() {
            return false;
        }

        if let Token::Literal(Literal::RegisterPair(_)) = self {
            return prefix.is_none() && suffix.is_none();
        }

        match self {
            Token::Literal(_) | Token::Symbol(_) => {
                if let Some(prefix) = prefix {
                    *prefix == Flag::N || *prefix == Flag::I
                } else {
                    true
                }
            }
            Token::Command(_) => {
                if let Some(prefix) = prefix {
                    *prefix == Flag::E
                } else {
                    *suffix == None
                }
            }
        }
    }

    pub fn from_lexemas(lexemas: &Vec<String>) -> Result<(Vec<Token>, Stat), Box<dyn Error>> {
        let mut res = vec![];
        let mut stat = Stat::default();

        for str in lexemas {
            let prefix_flag = Flag::from_prefix(str);
            let suffix_flag = Flag::from_suffix(str);
            let mut lexema: &str;
            let token: Token;

            // eliminate prefix and suffix
            if prefix_flag.is_some() {
                lexema = &str[1..];
            } else {
                lexema = &str;
            }

            if suffix_flag.is_some() {
                lexema = &lexema[..lexema.len() - 2];
            }


            // make token
            if let Some(command) = COMMANDS.get(lexema) {
                token = Token::Command(*command);
            } else {
                if Self::is_symbol(lexema) {
                    token = Token::Symbol(String::from(lexema));
                } else {
                    let lit = Literal::from_str(lexema);

                    match lit {
                        Ok(lit) => {
                            token = Token::Literal(lit);
                        }
                        Err(err) => {
                            return Err(err);
                        }

                    }
                }
            }

            if !token.is_valid(&prefix_flag, &suffix_flag) || !stat.is_valid() {
                 Err("Invalid Token")?
            }


            // set state
            if let Some(prefix) = prefix_flag {
                stat.set(prefix);
            }

            if let Some(suffix) = suffix_flag {
                stat.set(suffix);
            }

            res.push(token);
        }

        Ok((res, stat))
    }
}
