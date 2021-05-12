use anyhow::Context;
use lazy_static::lazy_static;
use std::collections::HashSet;

use crate::{lexer::LispNum, parser::Datum, CompilerError};

lazy_static! {
    static ref KEYWORDS: HashSet<&'static str> = [
        "lambda",
        "if",
        "quote",
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
    QuasiQuotation(Box<Datum>),
    ProcedureCall(ProcedureCall),
    Lambda(Lambda),
    Conditional(Conditional),
    Assignment(Assignment),
    // Derived // Implement later along with quasiquotations
    MacroUse(MacroUse),
    MacroBlock,
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

impl<'a> ScopedExpression<'a> {
    fn empty_scope(expression: Expression, parent_scope: &[&'a Scope]) -> Self {
        let current_scope = Scope::new();
        let parent_scope = parent_scope.iter().cloned().collect();
        Self {
            expression,
            current_scope,
            parent_scope,
        }
    }
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
                Ok(ScopedExpression::empty_scope(variable, parent_scope))
            }
        }
        Datum::Boolean(v) => {
            let boolean = Expression::SelfEvaluating(SelfEvaluating::Boolean(v));
            Ok(ScopedExpression::empty_scope(boolean, parent_scope))
        }
        Datum::Number(v) => {
            let number = Expression::SelfEvaluating(SelfEvaluating::Number(v));
            Ok(ScopedExpression::empty_scope(number, parent_scope))
        }
        _ => {
            return Err(CompilerError::SyntaxError);
        }
    }
}
