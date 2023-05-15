use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{error::Rich, prelude::*};

#[derive(Debug)]
pub enum Expr<'a> {
    Num(f64),
    Reference(&'a str),

    Neg(Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),

    Binding {
        name: &'a str,
        value: Box<Expr<'a>>,
    },
    LetIn {
        bindings: Vec<Expr<'a>>,
        body: Box<Expr<'a>>,
    },

    Call(&'a str, Vec<Expr<'a>>),
    Lambda {
        arg: &'a str,
        body: Box<Expr<'a>>,
    },
}

pub fn parser<'a>() -> impl Parser<'a, &'a str, Expr<'a>, extra::Err<Rich<'a, char>>> {
    let ident = text::ident().padded();

    let expr = recursive(|expr| {
        let int = text::int(10)
            .map(|s: &str| Expr::Num(s.parse().unwrap()))
            .padded();

        let call = ident
            .then(
                expr.clone()
                    .padded()
                    .repeated()
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .map(|(f, args)| Expr::Call(f, args));

        let atom = int
            .or(expr.delimited_by(just('('), just(')')))
            .or(call)
            .or(ident.map(Expr::Reference));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .foldr(atom, |_op, rhs| Expr::Neg(Box::new(rhs)));

        let product = unary.clone().foldl(
            choice((
                op('*').to(Expr::Mul as fn(_, _) -> _),
                op('/').to(Expr::Div as fn(_, _) -> _),
            ))
            .then(unary)
            .repeated(),
            |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
        );

        let sum = product.clone().foldl(
            choice((
                op('+').to(Expr::Add as fn(_, _) -> _),
                op('-').to(Expr::Sub as fn(_, _) -> _),
            ))
            .then(product)
            .repeated(),
            |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
        );

        sum
    });

    let decl = recursive(|decl| {
        let binding = ident
            .then_ignore(just('='))
            .then(decl.clone())
            .then_ignore(just(';'))
            .padded()
            .map(|(ident, expr)| Expr::Binding {
                name: ident,
                value: Box::new(expr),
            })
            .labelled("binding");

        let let_in = text::keyword("let")
            .ignore_then(binding.repeated().collect())
            .then_ignore(text::keyword("in"))
            .then(decl.clone())
            .map(|(bindings, body)| Expr::LetIn {
                bindings,
                body: Box::new(body),
            })
            .labelled("let-in");

        let func = ident
            .then_ignore(just(':'))
            .then(decl.clone())
            .map(|(ident, expr)| Expr::Lambda {
                arg: ident,
                body: Box::new(expr),
            });

        let_in.or(func).or(expr).padded()
    });

    decl
}

fn main() {
    let file_name = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(&file_name).unwrap();

    let (ast, errs) = parser().parse(&src).into_output_errors();
    if !errs.is_empty() {
        errs.into_iter()
            .map(|e| e.map_token(|c| c.to_string()))
            .for_each(|e| {
                Report::build(ReportKind::Error, file_name.clone(), e.span().start)
                    .with_message(e.to_string())
                    .with_label(
                        Label::new((file_name.clone(), e.span().into_range()))
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .with_labels(e.contexts().map(|(label, span)| {
                        Label::new((file_name.clone(), span.into_range()))
                            .with_message(format!("while parsing this {}", label))
                            .with_color(Color::Yellow)
                    }))
                    .finish()
                    .print(sources([(file_name.clone(), src.clone())]))
                    .unwrap()
            });
        std::process::exit(1);
    }

    println!("{:#?}", ast);
}
