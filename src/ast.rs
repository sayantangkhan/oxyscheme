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

enum Expression {
    Variable(Variable),
    SelfEvaluating(SelfEvaluating),
    Quotation(Box<Datum>),
    // QuasiQuotation(Box<Datum>),
    ProcedureCall(ProcedureCall),
    Lambda(Lambda),
    Conditional(Conditional),
    Assignment(Assignment),
    // Derived // Implement later along with quasiquotations
    MacroUse(MacroUse),
    MacroBlock,
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

struct ProcedureCall {
    operator: Box<Expression>,
    operand: Vec<Expression>,
}

struct Lambda {
    arguments: LambdaArgs,
    body: LambdaBody,
}

enum LambdaArgs {
    List(Vec<Variable>),
    Atom(Variable),
    Pair(Vec<Variable>, Variable),
}

struct LambdaBody {
    definitions: Vec<Definition>,
    commands: Vec<Expression>,
    return_expression: Box<Expression>,
}

enum Definition {
    DefVar(Variable, Box<Expression>),
    DefLambda(Variable, Vec<Variable>, LambdaBody),
    Multiple(Vec<Definition>),
}

struct Conditional {
    test: Box<Expression>,
    consequent: Box<Expression>,
    alternate: Option<Box<Expression>>,
}

struct Assignment {
    variable: Variable,
    expression: Box<Expression>,
}

struct MacroUse {
    keyword: String,
    datum: Vec<Datum>,
}

enum MacroBlock {
    Let(Syntax),
    LetRec(Syntax),
}

struct Syntax {
    syntax_spec: Vec<SyntaxSpec>,
    body: LambdaBody,
}

struct SyntaxSpec {
    keyword: String,
    transformer_spec: TransformerSpec,
}

struct TransformerSpec {
    identifiers: Vec<String>,
    syntax_rules: Vec<SyntaxRule>,
}

struct SyntaxRule {
    pattern: Pattern,
    template: Template,
}

enum Pattern {
    Identifier(String),
    List(Vec<Pattern>),
    Pair(Vec<Pattern>, Box<Pattern>),
    EllipsisList(Vec<Pattern>, Box<Pattern>),
    Vector(Vec<Pattern>),
    EllipsisVector(Vec<Pattern>, Box<Pattern>),
    Datum(SelfEvaluating),
}

enum Template {
    Identifier(String),
    List(Vec<TemplateElem>),
    Pair(Vec<TemplateElem>, Box<TemplateElem>),
    Vector(Vec<TemplateElem>),
    Datum(SelfEvaluating),
}

enum TemplateElem {
    Template(Template),
    TemplateEllipsis(Template),
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
        Datum::Quote(v) => {
            let quote = Expression::Quotation(v);
            Ok(quote.scopify_with_empty_scope(parent_scope))
        }
        // Datum::Backquote(v) => {
        //     let quasiquote = Expression::QuasiQuotation(v);
        //     Ok(quasiquote.scopify_with_empty_scope(parent_scope))
        // }
        Datum::List(contents) => parse_scoped_list(contents, parent_scope),
        _ => {
            return Err(CompilerError::SyntaxError);
        }
    }
}

fn parse_scoped_list<'a>(
    contents: Vec<Datum>,
    parent_scope: &[&'a Scope],
) -> Result<ScopedExpression<'a>, CompilerError> {
    let car = contents.get(0).ok_or(CompilerError::SyntaxError)?;
    let cdr = &contents[1..];

    match car {
        Datum::Identifier(v) if v == "quote" => {
            let datum_internal: Vec<Datum> = cdr.iter().cloned().collect();
            let datum = Box::new(Datum::List(datum_internal));
            Ok(Expression::Quotation(datum).scopify_with_empty_scope(parent_scope))
        }
        Datum::Identifier(v) if v == "lambda" => parse_lambda(cdr, parent_scope),
        _ => {
            return Err(CompilerError::SyntaxError);
        }
    }
}

fn parse_lambda<'a>(
    lambda_cdr: &[Datum],
    parent_scope: &[&'a Scope],
) -> Result<ScopedExpression<'a>, CompilerError> {
    let mut current_scope = Scope::new();

    let cadr = lambda_cdr.get(0).ok_or(CompilerError::SyntaxError)?;
    let mut cddr = lambda_cdr.into_iter();

    let lambda_args = match cadr {
        Datum::Identifier(variable_name) => {
            current_scope.variables.insert(variable_name.to_string());
            LambdaArgs::Atom(Variable {
                name: variable_name.to_string(),
            })
        }
        Datum::List(datum_list) => {
            let mut variables: Vec<Variable> = Vec::new();
            for datum in datum_list {
                match datum {
                    Datum::Identifier(variable_name) => {
                        current_scope.variables.insert(variable_name.to_string());
                        variables.push(Variable {
                            name: variable_name.to_string(),
                        });
                    }
                    _ => {
                        return Err(CompilerError::SyntaxError);
                    }
                }
            }
            LambdaArgs::List(variables)
        }
        Datum::DottedPair(datum_list, final_datum) => {
            let mut variables: Vec<Variable> = Vec::new();
            let mut final_variable;
            for datum in datum_list {
                match datum {
                    Datum::Identifier(variable_name) => {
                        current_scope.variables.insert(variable_name.to_string());
                        variables.push(Variable {
                            name: variable_name.to_string(),
                        });
                    }
                    _ => {
                        return Err(CompilerError::SyntaxError);
                    }
                }
            }

            match **final_datum {
                Datum::Identifier(ref variable_name) => {
                    current_scope.variables.insert(variable_name.to_string());
                    final_variable = Variable {
                        name: variable_name.to_string(),
                    };
                }
                _ => {
                    return Err(CompilerError::SyntaxError);
                }
            }

            LambdaArgs::Pair(variables, final_variable)
        }
        _ => {
            return Err(CompilerError::SyntaxError);
        }
    };

    let definitions = parse_scoped_definitions(&mut cddr, &mut current_scope, parent_scope)?;

    todo!()
}

fn parse_scoped_definitions<'a, I>(
    mut body: &mut I,
    current_scope: &mut Scope,
    parent_scope: &[&Scope],
) -> Result<Vec<Definition>, CompilerError>
where
    I: Iterator<Item = &'a Datum>,
{
    let mut definitions: Vec<Definition> = Vec::new();
    while let Some(datum) = body.next() {
        match datum {
            Datum::List(elements) => {
                let head = elements.get(0).ok_or(CompilerError::SyntaxError)?;
                if let Variable { name } = head && name == "define" {
                    todo!()
                }
            }
            _ => {
                break;
            }
        }
    }

    Ok(definitions)
}
