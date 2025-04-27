use crate::emitter::ast::*;

pub fn visit_block(statement_list: &Block) -> String {
    statement_list
        .stmts
        .iter()
        .map(|stmt| visit_statement(stmt))
        .collect::<Vec<_>>()
        .join("\n")
}

fn visit_statement(statement: &Statement) -> String {
    match statement {
        Statement::Expression(expr) => visit_expression(expr),
        Statement::VarDeclaration(VarDeclaration {
            name: ident,
            rhs: expr,
            declarion_type: assign_type,
        }) => {
            let s1 = visit_ident(ident);
            let s2 = visit_rhs_expression(expr);
            format!("{assign_type} {s1} = {s2}")
        }
        Statement::Assignment(Assignment {
            variable_name: ident,
            rhs: expr,
        }) => {
            let s1 = visit_ident(ident);
            let s2 = visit_rhs_expression(expr);
            format!("{s1} = {s2}")
        }
        Statement::FuncDeclaration(FuncDeclaration {
            name,
            args,
            return_type,
            body: stmts,
        }) => {
            let s_name = visit_ident(name);
            let s_args = args.iter().fold(String::new(), |mut acc, a| {
                let arg_name = visit_ident(&a.name);
                acc.push_str(arg_name.as_str());
                acc.push_str(" ");
                acc.push_str(&a._type.to_string());
                acc
            });

            let mut result = format!("{s_name} ({s_args})");
            if return_type.name != TypeName::Unit {
                let s_return_type = visit_type(return_type);
                result = format!("{result} {s_return_type}");
            }
            let s_stmts = visit_block(stmts).replace("\n", "\n\t");

            format!("func {result}{{\n\t{s_stmts} \n}}")
        }
        Statement::Block(stmts) => {
            format!("{{\n{}\n}}", visit_block(stmts))
        }
        Statement::ControlFlow(cf) => format!("{cf:#?}"),
        Statement::Loop(l) => {
            let s = format!("for {} in {}..{}", l.var.value, l.start, l.end);
            let body = visit_block(&l.body);

            s + " {\n" + &body + "\n}\n"
        }
    }
}

fn visit_rhs_expression(rhs_expression: &RhsExpression) -> String {
    match rhs_expression {
        RhsExpression::Expression(expr) => visit_expression(expr),
        RhsExpression::Block(block) => visit_block(block),
    }
}

fn visit_expression(expression: &Expression) -> String {
    match expression {
        Expression::Ident(ident) => visit_ident(ident),
        Expression::Literal(literal) => visit_literal(literal),
        Expression::Call(Call {
            func_name,
            ref args,
        }) => {
            let s1 = visit_ident(func_name);
            let s2 = args
                .iter()
                .map(|expr| visit_expression(expr))
                .collect::<Vec<String>>()
                .join(", ");
            s1 + "(" + &s2 + ")"
        }
        Expression::Unary(unary_operation) => match unary_operation {
            UnaryOperation::Minus(expr) => todo!(),
            UnaryOperation::Not(expr) => todo!(),
            UnaryOperation::Ref(expr) => format!("&{}", visit_expression(expr)),
        },
        Expression::Binary(binary_operation) => todo!(),
    }
}

fn visit_type(_type: &Type) -> String {
    format!("{_type}")
}

fn visit_literal(literal: &Literal) -> String {
    match literal {
        Literal::String(str) => format!(".{str}").replace("\n", "\\n"),
        Literal::Integer(number) => visit_number(number),
    }
}

fn visit_ident(ident: &Ident) -> String {
    ident.value.to_owned()
}

fn visit_number(number: &Integer) -> String {
    number.value.to_string()
}
