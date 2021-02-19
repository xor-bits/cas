use std::collections::HashMap;

use crate::{Operator, Symbol};

enum Pattern {
    Symbol(Symbol),
    Identifier(u32),
}

macro_rules! pattern {
    (op: $e:expr) => {
        Pattern::Symbol(Symbol::Operator(Operator::from($e).unwrap()))
    };
    (id: $e:expr) => {
        Pattern::Identifier($e)
    };
    (nu: $e:expr) => {
        Pattern::Symbol(Symbol::Number($e))
    };
}

macro_rules! patterns {
	($($names:ident: $es:expr),*) => {
		&[ $(pattern!($names: $es)),* ]
	};
}

macro_rules! simplifier_rule {
    (find: { $($names1:ident: $es1:expr),* },  repl: { $($names2:ident: $es2:expr),* }) => {
		(
			patterns!($($names1: $es1),*),
			patterns!($($names2: $es2),*)
		)
	};
}

pub struct Simplifier {
    rules: Vec<(&'static [Pattern], &'static [Pattern])>,
}

impl Simplifier {
    pub fn new() -> Self {
        let mut simplifier = Simplifier { rules: Vec::new() };

        // ----------------
        // simplifier rules
        // ----------------

        // x + 0 = x
        simplifier.rules.push(simplifier_rule!(
            find: { nu: 0.0, id: 0, op: '+' },
            repl: { id: 0 }
        ));
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, nu: 0.0, op: '+' },
            repl: { id: 0 }
        ));
        // x * 0 = 0
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, nu: 0.0, op: '*' },
            repl: {}
        ));
        simplifier.rules.push(simplifier_rule!(
            find: { nu: 0.0, id: 0, op: '*' },
            repl: {}
        ));
        // x * x = x^2
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 0, op: '*' },
            repl: { id: 0, nu: 2.0, op: '^' }
        ));
        // x + x = 2x
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 0, op: '+' },
            repl: { nu: 2.0, id: 0, op: '*' }
        ));
        // x / x = 1
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 0, op: '/' },
            repl: { nu: 1.0 }
        ));
        // x / y = x * y^-1
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 1, op: '/' },
            repl: { id: 0, id: 1, nu: -1.0, op: '^', op: '*' }
        ));
        // x + x = 2x
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 0, op: '+' },
            repl: { nu: 2.0, id: 0, op: '*' }
        ));
        // x + x * y = x * (1 + y)
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 0, id: 1, op: '*', op: '+' },
            repl: { id: 0, nu: 1.0, id: 1, op: '+', op: '*' }
        ));
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 1, op: '*', id: 0, op: '+' },
            repl: { id: 0, nu: 1.0, id: 1, op: '+', op: '*' }
        ));
        // x^y + x^z = x^(y+z)
        simplifier.rules.push(simplifier_rule!(
            find: { id: 0, id: 1, op: '^', id: 0, id: 2, op: '^', op: '+' },
            repl: { id: 0, id: 2, id: 2, op: '+', op: '^' }
        ));

        simplifier
    }
}
