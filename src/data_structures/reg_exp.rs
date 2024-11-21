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

    pub fn to_string(self) -> String {
        match self {
            RegularExpression::Zero => String::from("0"),
            RegularExpression::One => String::from("1"),
            RegularExpression::Letter(ch) => ch.to_string(),
            RegularExpression::Sum((lhs, rhs)) => {
                format!("({} + {})", lhs.to_string(), rhs.to_string())
            },
            RegularExpression::Concatenation((lhs, rhs)) => {
                format!("({}{})", lhs.to_string(), rhs.to_string())
            }
            RegularExpression::Iteration(val) => {
                format!("{}*", val.to_string())
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_fail1() {
        let _ = RegularExpression::from_reverse_polish(&"ab".to_string()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail2() {
        let _ = RegularExpression::from_reverse_polish(&"c++".to_string()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail3() {
        let _ = RegularExpression::from_reverse_polish(&"mmm...".to_string()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail4() {
        let _ = RegularExpression::from_reverse_polish(&"*iz*e**".to_string()).unwrap();
    }

    #[test]
    fn test_ok() {
        let _ = RegularExpression::from_reverse_polish(&"xy+z.*".to_string()).unwrap();
    }
}
