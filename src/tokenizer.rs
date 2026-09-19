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

    for c in input.chars() {
        match splited_qoutation.last().unwrap() {
            Quotation::No => {
                if c == '\'' {
                    splited_qoutation.push(Quotation::Single);
                    splited.push("".to_string());
                } else {
                    splited.last_mut().unwrap().push(c);
                }
            }
            Quotation::Single => {
                if c == '\'' {
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
