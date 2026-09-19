use std::iter::zip;

#[derive(Debug)]
enum Quotation {
    No,
    Single,
    //Double,
}

pub fn tokenize(input: &String) -> Vec<String> {
    let mut splited = vec!["".to_string()];
    let mut splited_qoutation = vec![Quotation::No];

    let mut input_iter = input.chars().peekable();
    //for c in input_iter {
    while let Some(c) = input_iter.next() {
        match splited_qoutation.last().unwrap() {
            Quotation::No => {
                if c == '\'' {
                    if input_iter.peek() == Some(&'\'') {
                        //Empty quotes '' are ignored.
                        _ = input_iter.next();
                        continue;
                    }
                    splited_qoutation.push(Quotation::Single);
                    splited.push("".to_string());
                } else {
                    splited.last_mut().unwrap().push(c);
                }
            }
            Quotation::Single => {
                if c == '\'' {
                    if input_iter.peek() == Some(&'\'') {
                        //Adjacent quoted strings are concatenated.
                        _ = input_iter.next();
                        continue;
                    }
                    splited_qoutation.push(Quotation::No);
                    splited.push("".to_string());
                } else {
                    splited.last_mut().unwrap().push(c);
                }
            }
        }
    }

    //println!("Split Vektor {:?}", splited); //DEBUG
    //println!("Quotation Vektor {:?}", splited_qoutation); //DEBUG

    let mut processed: Vec<String> = Vec::new();
    for (quot, ele) in zip(splited_qoutation, splited) {
        let mut ele_processed = match quot {
            Quotation::No => ele.split_whitespace().map(String::from).collect(),
            Quotation::Single => vec![ele],
        };
        processed.append(&mut ele_processed);
    }

    //println!("Final: {:?}", processed); //DEBUG
    processed

    //input.split_whitespace().map(String::from).collect()
}
