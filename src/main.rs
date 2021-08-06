#[macro_use]
extern crate clap;

use regex::Regex;
use std::fmt::{self, Display};
use std::str::FromStr;
use dialoguer::Input;

fn main() {
    let matches = clap_app!(calculator =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg expr: "The expression to be evaluated")
    )
    .get_matches();

    if let Some(expr) = matches.value_of("expr") {
        println!("{}", solve(expr));
    } else {
        console();
    }
}

fn solve(str: &str) -> f64 {
    let mut output: Vec<f64> = Vec::new();
    let mut operator_stack: Vec<Operator> = Vec::new();

    for token in tokenize(str) {
        match token {
            Token::Number(n) => output.push(n),
            Token::Operator(Operator::LPar) => operator_stack.push(Operator::LPar),
            Token::Operator(Operator::RPar) => {
                while first_operator_in_stack(&operator_stack) != Some(Operator::LPar) {
                    assert!(!operator_stack.is_empty(), "Mismatched parentheses");
                    output.apply(operator_stack.pop().unwrap());
                }

                assert_eq!(Some(Operator::LPar), operator_stack.pop());
            }
            Token::Operator(op) => {
                while let Some(stack_operator) = first_non_para_in_stack(&operator_stack) {
                    if stack_operator.precedence() > op.precedence()
                        || (stack_operator.precedence() == op.precedence()
                            && op.is_left_associative())
                    {
                        output.apply(operator_stack.pop().unwrap());
                    } else {
                        break;
                    }
                }

                operator_stack.push(op);
            }
        }
    }

    operator_stack.reverse();

    for operator in operator_stack {
        assert_ne!(Operator::LPar, operator, "Mismatched parentheses");
        output.apply(operator);
    }

    output[0]
}

#[derive(Clone, Copy, Debug)]
 enum Token {
    Number(f64),
    Operator(Operator),
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => n.fmt(f),
            Self::Operator(op) => op.fmt(f),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
 enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    LPar,
    RPar,
}

impl Operator {
    fn from_symbol(s: char) -> Self {
        match s {
            '+' => Self::Add,
            '-' => Self::Sub,
            '*' => Self::Mul,
            '/' => Self::Div,
            '%' => Self::Mod,
            '^' => Self::Exp,
            '(' => Self::LPar,
            ')' => Self::RPar,
            _ => panic!(),
        }
    }

    fn precedence(&self) -> usize {
        match self {
            Self::Add | Self::Sub => 2,
            Self::Mul | Self::Div | Self::Mod => 3,
            Self::Exp => 4,
            _ => 0,
        }
    }

    fn is_left_associative(&self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod
        )
    }

    fn apply(&self, lhs: f64, rhs: f64) -> f64 {
        match self {
            Self::Add => lhs + rhs,
            Self::Sub => lhs - rhs,
            Self::Mul => lhs * rhs,
            Self::Div => lhs / rhs,
            Self::Mod => lhs % rhs,
            Self::Exp => lhs.powf(rhs),
            _ => panic!(),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Exp => write!(f, "^"),
            Self::LPar => write!(f, "("),
            Self::RPar => write!(f, ")"),
        }
    }
}

 fn tokenize(str: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let regex = Regex::new(r"([0-9]+(.[0-9]+)*)|([+\-*/%^\(\)])").unwrap();

    for token in regex.captures_iter(str) {
        if let Some(number) = token.get(1).map(|m| m.as_str()) {
            let n = f64::from_str(number).unwrap();
            tokens.push(Token::Number(n));
        } else if let Some(operator) = token.get(3).map(|m| m.as_str()) {
            let op = Operator::from_symbol(operator.chars().next().unwrap());
            tokens.push(Token::Operator(op));
        }
    }

    tokens
}

fn first_operator_in_stack(stack: &[Operator]) -> Option<Operator> {
    if stack.is_empty() {
        return None;
    }

    Some(stack[stack.len() - 1])
}

fn first_non_para_in_stack(stack: &[Operator]) -> Option<Operator> {
    first_operator_in_stack(stack).and_then(|op| if op != Operator::LPar { Some(op) } else { None })
}

trait Apply {
    fn apply(&mut self, op: Operator);
}

impl Apply for Vec<f64> {
    fn apply(&mut self, op: Operator) {
        self.pop()
            .and_then(|x| self.pop().map(|y| self.push(op.apply(y, x))))
            .unwrap()
    }
}

fn console() {
    loop {
        let input: String = Input::new()
            .with_prompt("> ")
            .interact_text()
            .unwrap();

        let input: &str = &input;

        if matches!(input, "q" | "exit") {
            break;
        }

        println!("{}", solve(&input));
    }
}
