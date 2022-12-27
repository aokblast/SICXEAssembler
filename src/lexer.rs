pub fn parse_line_to_lexemes(line: &str) -> Vec<String> {
    let mut lexeme = String::new();
    let mut res= vec![];
    let mut in_str = false;

    for c in line.chars() {
        match c {
            '\'' => {
                lexeme.push(c);
                in_str = !in_str;
            }
            ' ' | '\t' => {
                if in_str {
                    lexeme.push(' ');
                } else if lexeme.len() > 0 {
                    res.push(lexeme);
                    lexeme = String::new();
                }
            }
            _ => {
                lexeme.push(c);
            }

        }
    }

    if lexeme.len() > 0 {
        res.push(lexeme);
    }

    res
}


