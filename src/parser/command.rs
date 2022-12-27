use lazy_static::lazy_static;
use std::collections::HashMap;


#[derive(PartialEq)]
pub enum Flag {
    N = 5,
    I = 4,
    X = 3,
    B = 2,
    P = 1,
    E = 0
}

impl Flag {
    pub fn from_prefix(lexeme: &str) -> Option<Flag> {
        return match lexeme.chars().nth(0).unwrap() {
            '+' => {
                Some(Flag::E)
            }
            '@' => {
                Some(Flag::N)
            }
            '#' => {
                Some(Flag::I)
            }
            _ => {
                None
            }
        }
    }

    pub fn from_suffix(lexeme: &str) -> Option<Flag> {
        if lexeme.len() > 2 && &lexeme[lexeme.len() - 2..] == ",X" {
            return Some(Flag::X);
        }
        None
    }
}


#[derive(Copy, Clone, Default)]
pub struct Stat(u8);


impl Stat {
    #[inline]
    pub fn set(self: &mut Self, flag: Flag) {
        self.0 |= 1 << (flag as i32);
    }

    #[inline]
    pub fn unset(self: &mut Self, flag: Flag) {
        self.0 ^= 1 << (flag as i32);
    }


    #[inline]
    pub fn is_set(self: &Self, flag: Flag) -> bool {
        (self.0 & 1 << (flag as i32)) != 0
    }

    pub fn is_valid(self: &Self) -> bool {
        // P and B and E cannot be all set up
        // N or I cannot be set with X
        !((self.is_set(Flag::P) ^ self.is_set(Flag::B) ^ self.is_set(Flag::E)) && (self.is_set(Flag::P) && self.is_set(Flag::B) && self.is_set(Flag::E)))
            && !(self.is_set(Flag::N) ^ self.is_set(Flag::I) && self.is_set(Flag::X))
    }


    #[inline]
    pub fn get_val(self: &Self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub enum Register {
    A = 0,
    X = 1,
    L = 2,
    B = 3,
    S = 4,
    T = 5,
    F = 6,
}

impl Register {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'A' => {
                Some(Self::A)
            }
            'X' => {
                Some(Self::X)
            }
            'L' => {
                Some(Self::L)
            }
            'B' => {
                Some(Self::B)
            }
            'S' => {
                Some(Self::S)
            }
            'T' => {
                Some(Self::T)
            }
            'F' => {
                Some(Self::F)
            }
            _ => {
                None
            }
        }

    }
}

#[derive(Copy, Clone)]
pub enum Format {
    ONE,
    TWO,
    ThreeAndFour
}

impl Format {
    pub fn len(self: &Self) -> usize {
        match self {
            Self::ONE => {
                3
            }
            Self::TWO => {
                2
            }
            Self::ThreeAndFour => {
                3
            }
        }
    }
}


#[derive(Copy, Clone, PartialEq)]
pub enum Directive {
    START,
    END,
    BYTE,
    WORD,
    RESB,
    RESW,
    BASE
}

impl Directive {
    pub fn len(self: &Self) -> usize {
        match self {
            Self::BASE | Self::START | Self::END => {
                0
            }
            Self::BYTE | Self::RESB => {
                1
            }
            Self::WORD | Self::RESW => {
                3
            }
        }
    }

}

#[derive(Copy, Clone)]
pub enum Command {
    Directive(Directive),
    Mnemonic(Mnemonic)
}

#[derive(Copy, Clone)]
pub struct Mnemonic {
    pub opcode: u8,
    pub format: Format,
}




lazy_static! {
    pub static ref COMMANDS: HashMap<&'static str, Command> = {
        let mut map = HashMap::new();
        map.insert("ADD",    Command::Mnemonic(Mnemonic{opcode: 0x18, format: Format::ThreeAndFour}));
        map.insert("ADDF",   Command::Mnemonic(Mnemonic{opcode: 0x58, format: Format::ThreeAndFour}));
        map.insert("ADDR",   Command::Mnemonic(Mnemonic{opcode: 0x90, format: Format::TWO}));
        map.insert("AND",    Command::Mnemonic(Mnemonic{opcode: 0x40, format: Format::ThreeAndFour}));
        map.insert("CLEAR",  Command::Mnemonic(Mnemonic{opcode: 0xB4, format: Format::TWO}));
        map.insert("COMP",   Command::Mnemonic(Mnemonic{opcode: 0x28, format: Format::ThreeAndFour}));
        map.insert("COMPF",  Command::Mnemonic(Mnemonic{opcode: 0x88, format: Format::ThreeAndFour}));
        map.insert("COMPR",  Command::Mnemonic(Mnemonic{opcode: 0xA0, format: Format::TWO}));
        map.insert("DIV",    Command::Mnemonic(Mnemonic{opcode: 0x24, format: Format::ThreeAndFour}));
        map.insert("DIVF",   Command::Mnemonic(Mnemonic{opcode: 0x64, format: Format::ThreeAndFour}));
        map.insert("DIVR",   Command::Mnemonic(Mnemonic{opcode: 0x9C, format: Format::TWO}));
        map.insert("FIX",    Command::Mnemonic(Mnemonic{opcode: 0xC4, format: Format::ONE}));
        map.insert("FLOAT",  Command::Mnemonic(Mnemonic{opcode: 0xC0, format: Format::ONE}));
        map.insert("HIO",    Command::Mnemonic(Mnemonic{opcode: 0xF4, format: Format::ONE}));
        map.insert("J",      Command::Mnemonic(Mnemonic{opcode: 0x3C, format: Format::ThreeAndFour}));
        map.insert("JEQ",    Command::Mnemonic(Mnemonic{opcode: 0x30, format: Format::ThreeAndFour}));
        map.insert("JGT",    Command::Mnemonic(Mnemonic{opcode: 0x34, format: Format::ThreeAndFour}));
        map.insert("JLT",    Command::Mnemonic(Mnemonic{opcode: 0x38, format: Format::ThreeAndFour}));
        map.insert("JSUB",   Command::Mnemonic(Mnemonic{opcode: 0x48, format: Format::ThreeAndFour}));
        map.insert("LDA",    Command::Mnemonic(Mnemonic{opcode: 0x00, format: Format::ThreeAndFour}));
        map.insert("LDB",    Command::Mnemonic(Mnemonic{opcode: 0x68, format: Format::ThreeAndFour}));
        map.insert("LDCH",   Command::Mnemonic(Mnemonic{opcode: 0x50, format: Format::ThreeAndFour}));
        map.insert("LDF",    Command::Mnemonic(Mnemonic{opcode: 0x70, format: Format::ThreeAndFour}));
        map.insert("LDL",    Command::Mnemonic(Mnemonic{opcode: 0x08, format: Format::ThreeAndFour}));
        map.insert("LDS",    Command::Mnemonic(Mnemonic{opcode: 0x6C, format: Format::ThreeAndFour}));
        map.insert("LDT",    Command::Mnemonic(Mnemonic{opcode: 0x74, format: Format::ThreeAndFour}));
        map.insert("LDX",    Command::Mnemonic(Mnemonic{opcode: 0x04, format: Format::ThreeAndFour}));
        map.insert("LPS",    Command::Mnemonic(Mnemonic{opcode: 0xD0, format: Format::ThreeAndFour}));
        map.insert("MUL",    Command::Mnemonic(Mnemonic{opcode: 0x20, format: Format::ThreeAndFour}));
        map.insert("MULF",   Command::Mnemonic(Mnemonic{opcode: 0x60, format: Format::ThreeAndFour}));
        map.insert("MULR",   Command::Mnemonic(Mnemonic{opcode: 0x98, format: Format::TWO}));
        map.insert("NORM",   Command::Mnemonic(Mnemonic{opcode: 0xC8, format: Format::ONE}));
        map.insert("OR",     Command::Mnemonic(Mnemonic{opcode: 0x44, format: Format::ThreeAndFour}));
        map.insert("RD",     Command::Mnemonic(Mnemonic{opcode: 0xD8, format: Format::ThreeAndFour}));
        map.insert("RMO",    Command::Mnemonic(Mnemonic{opcode: 0xAC, format: Format::TWO}));
        map.insert("RSUB",   Command::Mnemonic(Mnemonic{opcode: 0x4C, format: Format::ONE}));
        map.insert("SHIFTL", Command::Mnemonic(Mnemonic{opcode: 0xA4, format: Format::TWO}));
        map.insert("SHIFTR", Command::Mnemonic(Mnemonic{opcode: 0xA8, format: Format::TWO}));
        map.insert("SIO",    Command::Mnemonic(Mnemonic{opcode: 0xF0, format: Format::ONE}));
        map.insert("SSK",    Command::Mnemonic(Mnemonic{opcode: 0xEC, format: Format::ThreeAndFour}));
        map.insert("STA",    Command::Mnemonic(Mnemonic{opcode: 0x0C, format: Format::ThreeAndFour}));
        map.insert("STB",    Command::Mnemonic(Mnemonic{opcode: 0x78, format: Format::ThreeAndFour}));
        map.insert("STCH",   Command::Mnemonic(Mnemonic{opcode: 0x54, format: Format::ThreeAndFour}));
        map.insert("STF",    Command::Mnemonic(Mnemonic{opcode: 0x80, format: Format::ThreeAndFour}));
        map.insert("STI",    Command::Mnemonic(Mnemonic{opcode: 0xD4, format: Format::ThreeAndFour}));
        map.insert("STL",    Command::Mnemonic(Mnemonic{opcode: 0x14, format: Format::ThreeAndFour}));
        map.insert("STS",    Command::Mnemonic(Mnemonic{opcode: 0x7C, format: Format::ThreeAndFour}));
        map.insert("STSW",   Command::Mnemonic(Mnemonic{opcode: 0xE8, format: Format::ThreeAndFour}));
        map.insert("STT",    Command::Mnemonic(Mnemonic{opcode: 0x84, format: Format::ThreeAndFour}));
        map.insert("STX",    Command::Mnemonic(Mnemonic{opcode: 0x10, format: Format::ThreeAndFour}));
        map.insert("SUB",    Command::Mnemonic(Mnemonic{opcode: 0x1C, format: Format::ThreeAndFour}));
        map.insert("SUBF",   Command::Mnemonic(Mnemonic{opcode: 0x5C, format: Format::ThreeAndFour}));
        map.insert("SUBR",   Command::Mnemonic(Mnemonic{opcode: 0x94, format: Format::TWO}));
        map.insert("SVC",    Command::Mnemonic(Mnemonic{opcode: 0xB0, format: Format::TWO}));
        map.insert("TD",     Command::Mnemonic(Mnemonic{opcode: 0xE0, format: Format::ThreeAndFour}));
        map.insert("TIO",    Command::Mnemonic(Mnemonic{opcode: 0xF8, format: Format::ONE}));
        map.insert("TIX",    Command::Mnemonic(Mnemonic{opcode: 0x2C, format: Format::ThreeAndFour}));
        map.insert("TIXR",   Command::Mnemonic(Mnemonic{opcode: 0xB8, format: Format::TWO}));
        map.insert("WD",     Command::Mnemonic(Mnemonic{opcode: 0xDC, format: Format::ThreeAndFour}));
        map.insert("START",  Command::Directive(Directive::START));
        map.insert("END",    Command::Directive(Directive::END));
        map.insert("BYTE",   Command::Directive(Directive::BYTE));
        map.insert("WORD",   Command::Directive(Directive::WORD));
        map.insert("RESB",   Command::Directive(Directive::RESB));
        map.insert("RESW",   Command::Directive(Directive::RESW));
        map.insert("BASE",   Command::Directive(Directive::BASE));
        map
    };
}