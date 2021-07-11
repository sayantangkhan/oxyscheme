// use anyhow::Context;
use lazy_static::lazy_static;
use std::collections::HashSet;

use crate::{lexer::LispNum, parser::Datum, CompilerError};

lazy_static! {
    static ref KEYWORDS: HashSet<&'static str> = [
        "lambda",
        "if",
        "quote",
        "begin",
        "set!",
        "let-syntax",
        "letrec-syntax",
        "syntax-rules",
        "define",
        "define-syntax"
    ]
    .iter()
    .cloned()
    .collect();
}

struct Scope {
    variables: HashSet<String>,
    macros: HashSet<String>,
}

impl Scope {
    fn new() -> Self {
        let variables = HashSet::new();
        let macros = HashSet::new();
        Self { variables, macros }
    }
}

struct ScopedExpression<'a> {
    expression: Expression,
    current_scope: Scope,
    parent_scope: Vec<&'a Scope>,
}

enum ExpOrDef {
    Expression(Expression),
    Definition,
    // SyntaxDefinition,
    Begin(Vec<ExpOrDef>),
}

enum Expression {
    Variable(Variable),
    SelfEvaluating(SelfEvaluating),
    Assignment(Assignment),
    // Quotation(Box<Datum>),
    // QuasiQuotation(Box<Datum>),
    // ProcedureCall(ProcedureCall),
    // Lambda(Lambda),
    // Conditional(Conditional),
    // Derived // Implement later along with quasiquotations
    // MacroUse(MacroUse),
    // MacroBlock,
}

impl<'a> Expression {
    fn scopify_with_empty_scope(self, parent_scope: &[&'a Scope]) -> ScopedExpression<'a> {
        let current_scope = Scope::new();
        let parent_scope = parent_scope.iter().cloned().collect();
        ScopedExpression {
            expression: self,
            current_scope,
            parent_scope,
        }
    }
}

struct Variable {
    name: String,
}

enum SelfEvaluating {
    Boolean(bool),
    Number(LispNum),
    Character(char),
    String(String),
}

struct Assignment {
    variable: Variable,
    expression: Box<Expression>,
}

fn parse_scoped_expression<'a>(
    datum: Datum,
    parent_scope: &[&'a Scope],
) -> Result<ScopedExpression<'a>, CompilerError> {
    match datum {
        Datum::Identifier(s) => {
            if KEYWORDS.contains(&s.as_str()) {
                return Err(CompilerError::SyntaxError); // TODO: Improve error message
            } else {
                let variable = Expression::Variable(Variable { name: s });
                Ok(variable.scopify_with_empty_scope(parent_scope))
            }
        }
        Datum::Boolean(v) => {
            let boolean = Expression::SelfEvaluating(SelfEvaluating::Boolean(v));
            Ok(boolean.scopify_with_empty_scope(parent_scope))
        }
        Datum::Number(v) => {
            let number = Expression::SelfEvaluating(SelfEvaluating::Number(v));
            Ok(number.scopify_with_empty_scope(parent_scope))
        }
        Datum::Character(v) => {
            let character = Expression::SelfEvaluating(SelfEvaluating::Character(v));
            Ok(character.scopify_with_empty_scope(parent_scope))
        }
        Datum::String(v) => {
            let string = Expression::SelfEvaluating(SelfEvaluating::String(v));
            Ok(string.scopify_with_empty_scope(parent_scope))
        }
        // Datum::Quote(v) => {
        // let quote = Expression::Quotation(v);
        // Ok(quote.scopify_with_empty_scope(parent_scope))
        // }
        // Datum::Backquote(v) => {
        //     let quasiquote = Expression::QuasiQuotation(v);
        //     Ok(quasiquote.scopify_with_empty_scope(parent_scope))
        // }
        // Datum::List(contents) => parse_scoped_list(contents, parent_scope),
        _ => {
            return Err(CompilerError::SyntaxError);
        }
    }
}

fn parse_scoped_list<'a>(
    mut contents: Vec<Datum>,
    parent_scope: &[&'a Scope],
) -> Result<ScopedExpression<'a>, CompilerError> {
    let car = contents.get(0).ok_or(CompilerError::SyntaxError)?;
    // let cdr = &contents[1..];

    match car {
        Datum::Identifier(v) if v == "set!" => {
            if contents.len() != 3 {
                return Err(CompilerError::SyntaxError);
            }
            let cdr1 = contents.pop().unwrap();
            let cdr0 = contents.get(1).unwrap();

            match cdr0 {
                Datum::Identifier(v) => {
                    // Add check for presence in scope
                    let variable = Variable {
                        name: v.to_string(),
                    };
                    let expression = parse_scoped_expression(cdr1, parent_scope)?.expression;
                    let assignment = Assignment {
                        variable,
                        expression: Box::new(expression),
                    };
                    Ok(Expression::Assignment(assignment).scopify_with_empty_scope(parent_scope))
                }
                _ => {
                    return Err(CompilerError::SyntaxError);
                }
            }
        }
        // Datum::Identifier(v) if v == "lambda" => parse_lambda(cdr, parent_scope),
        _ => {
            return Err(CompilerError::SyntaxError);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::parse_scoped_expression, lexer::LispNum, parser::*};

    #[test]
    fn test_assignment() {
        let input_datum = Datum::List(vec![
            Datum::Identifier("set!".to_string()),
            Datum::Identifier("x".to_string()),
            Datum::Number(LispNum::Integer(1)),
        ]);
        let parent_scope = &[];
        parse_scoped_expression(input_datum, parent_scope).unwrap();
    }
}
