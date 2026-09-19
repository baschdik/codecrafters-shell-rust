#[derive(Debug)]
enum Quotation {
    No,
    Single,
    Double,
}

pub fn tokenize(input: &String) -> Vec<String> {
    let mut splited = vec!["".to_string()];
    let mut splited_qoutation = vec![Quotation::No];

    let mut input_iter = input.chars().peekable();
    while let Some(c) = input_iter.next() {
        match splited_qoutation.last().unwrap() {
            Quotation::No => {
                if c.is_whitespace() {
                    splited.push("".to_string());
                    //remove extra whitespace:
                    while input_iter.next_if(|c| c.is_whitespace()).is_some() {}
                } else if c == '\'' {
                    //Ignore Empty quotes '':
                    if input_iter.peek() == Some(&'\'') {
                        _ = input_iter.next();
                        continue;
                    }
                    splited_qoutation.push(Quotation::Single);
                } else if c == '"' {
                    //Ignore Empty quotes "":
                    if input_iter.peek() == Some(&'"') {
                        _ = input_iter.next();
                        continue;
                    }
                    splited_qoutation.push(Quotation::Double);
                } else {
                    splited.last_mut().unwrap().push(c);
                }
            }
            Quotation::Single => {
                if c == '\'' {
                    //Concatenate adjacent quoted strings:
                    if input_iter.peek() == Some(&'\'') {
                        _ = input_iter.next();
                        continue;
                    }
                    splited_qoutation.push(Quotation::No);
                } else {
                    splited.last_mut().unwrap().push(c);
                }
            }
            Quotation::Double => {
                if c == '"' {
                    //Concatenate adjacent quoted strings:
                    if input_iter.peek() == Some(&'"') {
                        _ = input_iter.next();
                        continue;
                    }
                    splited_qoutation.push(Quotation::No);
                } else {
                    splited.last_mut().unwrap().push(c);
                }
            }
        }
    }

    println!("Final: {:?}", splited); //DEBUG
    splited
}
