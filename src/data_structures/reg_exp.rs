pub enum RegularExpression {
    Zero,
    One,
    Letter(char),
    Sum((Box<RegularExpression>, Box<RegularExpression>)),
    Concatenation((Box<RegularExpression>, Box<RegularExpression>)),
    Iteration(Box<RegularExpression>)
}

impl RegularExpression {
    pub fn from_reverse_polish(str: &String) -> Result<RegularExpression, String> {
        let mut stack = Vec::<RegularExpression>::new();
        for ch in str.chars() {
            match ch {
                '0' => {
                    stack.push(RegularExpression::Zero);
                }
                '1' => {
                    stack.push(RegularExpression::One);
                }
                '+' => {
                    if stack.len() < 2 {
                        return Err(String::from("Parsing error"));
                    }
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    stack.push(RegularExpression::Sum((Box::from(lhs), Box::from(rhs))));
                }
                '.' => {
                    if stack.len() < 2 {
                        return Err(String::from("Parsing error"));
                    }
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    stack.push(RegularExpression::Concatenation((Box::from(lhs), Box::from(rhs))));
                }
                '*' => {
                    if stack.is_empty() {
                        return Err(String::from("Parsing error"));
                    }
                    let val = stack.pop().unwrap();
                    stack.push(RegularExpression::Iteration(Box::from(val)));
                }
                _ => {
                    stack.push(RegularExpression::Letter(ch));
                }
            }
        }
        if stack.len() != 1 {
            Err(String::from("Parsing error"))
        } else {
            Ok(stack.pop().unwrap())
        }
    }
}