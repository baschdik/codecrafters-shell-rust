#[derive(Debug)]
enum Quotation {
    No,
    Single,
    Double,
}

pub fn tokenize(input: &String) -> Vec<String> {
    let mut processed = vec!["".to_string()];
    let mut qoutation_status = Quotation::No;

    let mut input_iter = input.chars().peekable();
    while let Some(c) = input_iter.next() {
        match qoutation_status {
            Quotation::No => {
                if c.is_whitespace() {
                    processed.push("".to_string());
                    //remove extra whitespace:
                    while input_iter.next_if(|c| c.is_whitespace()).is_some() {}
                } else if c == '\'' {
                    //Ignore Empty quotes '':
                    if input_iter.next_if(|c| *c == '\'').is_some() {
                        continue;
                    }

                    qoutation_status = Quotation::Single;
                } else if c == '"' {
                    //Ignore Empty quotes "":
                    if input_iter.next_if(|c| *c == '"').is_some() {
                        continue;
                    }
                    qoutation_status = Quotation::Double;
                } else {
                    processed.last_mut().unwrap().push(c);
                }
            }
            Quotation::Single => {
                if c == '\'' {
                    //Concatenate adjacent quoted strings:
                    if input_iter.next_if(|c| *c == '\'').is_some() {
                        continue;
                    }
                    qoutation_status = Quotation::No;
                } else {
                    processed.last_mut().unwrap().push(c);
                }
            }
            Quotation::Double => {
                if c == '"' {
                    //Concatenate adjacent quoted strings:
                    if input_iter.next_if(|c| *c == '\'').is_some() {
                        continue;
                    }
                    qoutation_status = Quotation::No;
                } else {
                    processed.last_mut().unwrap().push(c);
                }
            }
        }
    }

    println!("Final: {:?}", processed); //DEBUG
    processed
}
