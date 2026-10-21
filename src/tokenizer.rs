#[derive(Debug)]
enum Quotation {
    No,
    Single,
    Double,
}

pub fn tokenize(input: &str) -> Vec<String> {
    let mut processed = vec!["".to_string()];
    let mut qoutation_status = Quotation::No;

    let mut input_iter = input.chars().peekable();
    while let Some(c) = input_iter.next() {
        match qoutation_status {
            Quotation::No => {
                if c == '\\' {
                    //escaping
                    processed
                        .last_mut()
                        .unwrap()
                        .push(input_iter.next().unwrap_or_default());
                } else if c.is_whitespace() {
                    processed.push("".to_string());
                } else if c == '|' || c == ';' {
                    processed.push(c.to_string());
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
                if c == '\\' {
                    //Escape only " and \
                    processed.last_mut().unwrap().push(
                        input_iter
                            .next_if(|c| matches!(c, '"' | '\\'))
                            .unwrap_or_default(),
                    );
                } else if c == '"' {
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

    processed = processed
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| c.to_string())
        .collect();
    //println!("Final: {:?}", processed); //DEBUG
    processed
}
