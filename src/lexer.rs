pub fn parse_line_to_lexemas(line: &str) -> Vec<String> {
    let mut lexema = String::new();
    let mut res= vec![];
    let mut in_str = false;

    for c in line.chars() {
        match c {
            '\'' => {
                lexema.push(c);
                in_str = !in_str;
            }
            ' ' | '\t' => {
                if in_str {
                    lexema.push(' ');
                } else if lexema.len() > 0 {
                    res.push(lexema);
                    lexema = String::new();
                }
            }
            _ => {
                lexema.push(c);
            }

        }
    }

    if lexema.len() > 0 {
        res.push(lexema);
    }

    res
}


