use std::u32;

use crate::Associativity;

use super::{Engine, Expr, Operator, SymErr, Symbol, Tree};

fn split_keep<'a>(str: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in str.match_indices(|c| Operator::is_operator(c) || c == ',') {
        if last != index {
            result.push(&str[last..index]);
        }
        if matched != "," {
            result.push(matched);
        }
        last = index + matched.len();
    }
    if last < str.len() {
        result.push(&str[last..]);
    }

    result
}

pub fn parse_infix(engine: &Engine, infix_string: &str) -> Result<Vec<Symbol>, SymErr> {
    let py_fixed_str = infix_string.replace("**", "^").replace(' ', "");
    let infix_split = split_keep(py_fixed_str.as_str());

    if engine.debugging {
        println!("To infix: {}", infix_string);
    }

    Ok(infix_split
        .iter()
        .enumerate()
        .filter_map(|(i, &to_parse)| {
            if let Ok(number) = to_parse.parse() {
                Some(Ok(Symbol::Number(number)))
            } else if let Ok(oper) = Operator::from(to_parse.chars().next().unwrap()) {
                let is_sign = i == 0
                    || infix_split[i - 1] == "("
                    || infix_split[i - 1] == "-"
                    || infix_split[i - 1] == "+";
                if is_sign && oper == Operator::Add {
                    None
                } else if is_sign && oper == Operator::Sub {
                    Some(Ok(Symbol::Negate(())))
                } else if is_sign && !oper.is_parenthesis() {
                    Some(Err(SymErr::InvalidSign))
                } else {
                    Some(Ok(Symbol::Operator(oper)))
                }
            } else {
                if engine.functions.contains_key(to_parse) {
                    Some(Ok(Symbol::Function(String::from(to_parse))))
                } else {
                    Some(Ok(Symbol::Variable(String::from(to_parse))))
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn to_postfix(engine: &Engine, infix: &Vec<Symbol>) -> Result<Vec<Symbol>, SymErr> {
    let mut postfix = Vec::new();
    let mut operator_stack = Vec::new();
    let mut last_was_number = false;

    if engine.debugging {
        println!("To postfix: {:?}", infix);
    }

    for symbol in infix.iter() {
        last_was_number = match symbol {
            Symbol::Number(_) => {
                postfix.push(symbol.clone());
                true
            }
            Symbol::Variable(_) => {
                postfix.push(symbol.clone());
                false
            }
            Symbol::Function(_) => {
                operator_stack.push(symbol.clone());
                false
            }
            Symbol::Operator(Operator::LPa) => {
                if last_was_number {
                    operator_stack.push(Symbol::Operator(Operator::Mul));
                }
                operator_stack.push(symbol.clone());
                false
            }
            Symbol::Operator(Operator::RPa) => {
                loop {
                    let top_symbol = match operator_stack.pop() {
                        Some(Symbol::Operator(top_operator)) => {
                            if top_operator == Operator::LPa {
                                break;
                            }
                            Symbol::Operator(top_operator)
                        }
                        Some(Symbol::Negate(_)) => Symbol::Negate(()),
                        Some(sym) => sym.clone(),
                        None => return Err(SymErr::ParenthesesMismatch),
                    };

                    postfix.push(top_symbol.clone());
                }

                match operator_stack.pop() {
                    Some(Symbol::Function(string)) => {
                        postfix.push(Symbol::Function(string.clone()))
                    }
                    Some(symbol) => operator_stack.push(symbol),
                    None => (),
                }
                false
            }
            _ => {
                let mut operator = match symbol {
                    Symbol::Operator(o) => *o,
                    Symbol::Negate(_) => Operator::Custom(4, Associativity::Left),
                    _ => panic!("How did we get here?"),
                };

                loop {
                    let top_symbol = operator_stack.pop();
                    let top_operator = match top_symbol {
                        Some(Symbol::Operator(oper)) => oper,
                        Some(Symbol::Negate(_)) => Operator::Custom(4, Associativity::Left),
                        _ => break,
                    };

                    if top_operator.is_parenthesis()
                        || top_operator.precedence().unwrap() < operator.precedence().unwrap()
                    {
                        operator_stack.push(top_symbol.unwrap());
                        break;
                    }

                    operator = top_operator;
                    match top_operator {
                        Operator::Custom(_, _) => postfix.push(Symbol::Negate(())),
                        _ => postfix.push(Symbol::Operator(top_operator)),
                    }
                }
                operator_stack.push(symbol.clone());
                false
            }
        }
    }

    loop {
        match operator_stack.pop() {
            Some(symbol) => postfix.push(symbol.clone()),
            _ => break,
        }
    }

    Ok(postfix)
}

pub fn postfix_to_tree(engine: &Engine, postfix: &Vec<Symbol>) -> Result<Expr, SymErr> {
    let mut mixed_stack = Vec::new();

    if engine.debugging {
        println!("To expr tree: {:?}", postfix);
    }

    for symbol in postfix.iter() {
        match symbol {
            Symbol::Operator(op) => {
                let a = mixed_stack.pop().ok_or(SymErr::StackEmpty)?;
                let b = mixed_stack.pop().ok_or(SymErr::StackEmpty)?;

                mixed_stack.push(Expr::Operator(Tree {
                    value: op.clone(),
                    next: Some(vec![Box::new(b), Box::new(a)]),
                }));
            }
            Symbol::Negate(_) => {
                let a = mixed_stack.pop().ok_or(SymErr::StackEmpty)?;

                mixed_stack.push(Expr::Negate(Tree {
                    value: (),
                    next: Some(vec![Box::new(a)]),
                }));
            }
            Symbol::Number(n) => {
                mixed_stack.push(Expr::Number(n.clone()));
            }
            Symbol::Variable(s) => {
                mixed_stack.push(Expr::Variable(s.clone()));
            }
            Symbol::Function(s) => {
                let &(argc, _) = engine
                    .functions
                    .get(s.as_str())
                    .ok_or(SymErr::UnknownFunction)?;

                let mut arguments = Vec::new();
                for _ in 0..argc {
                    arguments.push(Box::new(
                        mixed_stack.pop().ok_or(SymErr::InvalidFunctionArgCount)?,
                    ));
                }

                mixed_stack.push(Expr::Function(Tree {
                    value: s.clone(),
                    next: Some(arguments),
                }));
            }
        }
    }

    assert!(mixed_stack.len() == 1, "Postfix had leftover symbols");

    Ok(mixed_stack.pop().unwrap())
}

pub fn tree_to_infix_recurse(expr: &Expr) -> (String, u8) {
    match &expr {
        Expr::Function(f) => (format!("{}(...)", f.value), u8::MAX),
        Expr::Operator(o) => {
            let a = tree_to_infix_recurse(&o.next.as_ref().unwrap()[0]);
            let b = tree_to_infix_recurse(&o.next.as_ref().unwrap()[1]);
            let c = o.value.precedence().unwrap();

            if c <= a.1 && c <= b.1 {
                (format!("{}{}{}", a.0, o.value.to(), b.0), c)
            } else if a.1 >= b.1 {
                (format!("{}{}({})", a.0, o.value.to(), b.0), c)
            } else {
                (format!("({}){}{}", a.0, o.value.to(), b.0), c)
            }
        }
        Expr::Negate(n) => (
            format!(
                "-({})",
                tree_to_infix_recurse(&n.next.as_ref().unwrap()[0]).0
            ),
            4,
        ),
        Expr::Variable(v) => (format!("{}", v), u8::MAX),
        Expr::Number(n) => (format!("{}", n), u8::MAX),
        Expr::Identifier(i) => (format!("i:{}", i), u8::MAX),
    }
}

pub fn tree_to_infix(expr: &Expr) -> String {
    tree_to_infix_recurse(expr).0
}
