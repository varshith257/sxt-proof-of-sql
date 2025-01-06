use alloc::{boxed::Box, vec, vec::Vec};
use proof_of_sql_parser::{
    intermediate_ast::{
        AggregationOperator, Expression, OrderBy, OrderByDirection, SelectResultExpr,
        SetExpression, Slice, TableExpression,
    },
    Identifier, SelectStatement,
};
use serde::{Deserialize, Serialize};
use sqlparser::ast::{BinaryOperator, Expr, Ident, UnaryOperator, Value};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// Represents an expression with an alias, e.g., `a + 1 AS b`
pub struct AliasedResultExpr {
    /// The underlying expression
    pub expr: Expr,
    /// The alias for the expression
    pub alias: Ident,
}

impl AliasedResultExpr {
    /// Create a new `AliasedResultExpr`
    pub fn new(expr: Expr, alias: Ident) -> Self {
        Self { expr, alias }
    }

    /// Extract the identifier from the expression if it is a column
    pub fn try_as_identifier(&self) -> Option<&Ident> {
        if let Expr::Identifier(ref ident) = self.expr {
            Some(ident)
        } else {
            None
        }
    }
}

///
/// # Panics
///
/// This function will panic if`name`(if provided) cannot be parsed.
/// Construct an identifier from a str
#[must_use]
pub fn ident(name: &str) -> Identifier {
    name.parse().unwrap()
}

// fn new_aliased_expr(expr: Expr, alias: &str) -> Expr {
//     Expr::Alias {
//         expr: Box::new(expr),
//         alias: Ident::new(alias),
//     }
// }

/// Construct a new boxed `Expression` A == B
#[must_use]
pub fn equal(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Eq,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` A >= B
#[must_use]
pub fn ge(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::GtEq,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` A <= B
#[must_use]
pub fn le(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::LtEq,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` NOT P
#[must_use]
pub fn not(expr: Expr) -> Expr {
    Expr::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(expr),
    }
}

/// Construct a new boxed `Expression` P AND Q
#[must_use]
pub fn and(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::And,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` P OR Q
#[must_use]
pub fn or(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Or,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` A + B
#[must_use]
pub fn add(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Plus,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` A - B
#[must_use]
pub fn sub(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Minus,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` A * B
#[must_use]
pub fn mul(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new boxed `Expression` A / B
#[must_use]
pub fn div(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Divide,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Get table from schema and name.
///
/// If the schema is `None`, the table is assumed to be in the default schema.
/// # Panics
///
/// This function will panic if either the `name` or the `schema` (if provided) cannot be parsed as valid [Identifier]s.
#[must_use]
pub fn tab(schema: Option<&str>, name: &str) -> Box<TableExpression> {
    Box::new(TableExpression::Named {
        table: name.parse().unwrap(),
        schema: schema.map(|schema| schema.parse().unwrap()),
    })
}

/// Get column from name
///
/// # Panics
///
/// This function will panic if the `name` cannot be parsed into a valid column expression as valid [Identifier]s.
#[must_use]
pub fn col(name: &str) -> Expr {
    Expr::Identifier(Ident::new(name))
}

/// Get literal from value
pub fn lit<L: Into<Value>>(literal: L) -> Expr {
    Expr::Value(literal.into())
}

/// Compute the sum of an expression
#[must_use]
pub fn sum(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Sum,
        expr,
    })
}

/// Compute the minimum of an expression
#[must_use]
pub fn min(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Min,
        expr,
    })
}

/// Compute the maximum of an expression
#[must_use]
pub fn max(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Max,
        expr,
    })
}

/// Count the amount of non-null entries of expression
#[must_use]
pub fn count(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Count,
        expr,
    })
}

/// Count the rows
#[must_use]
pub fn count_all() -> Box<Expression> {
    count(Box::new(Expression::Wildcard))
}

/// An expression with an alias i.e. EXPR AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed as valid [Identifier]s.
#[must_use]
pub fn aliased_expr(expr: Box<Expr>, alias: &str) -> AliasedResultExpr {
    AliasedResultExpr {
        expr: *expr,
        alias: alias.into(),
    }
}

/// Select all columns from a table i.e. SELECT *
#[must_use]
pub fn col_res_all() -> SelectResultExpr {
    SelectResultExpr::ALL
}

// /// Select one column from a table and give it an alias i.e. SELECT COL AS ALIAS
// ///
// /// # Panics
// ///
// /// This function will panic if the `alias` cannot be parsed as valid [Identifier]s.
// #[must_use]
// pub fn col_res(col_val: Box<Expr>, alias: &str) -> SelectResultExpr {
//     SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
//         expr: *col_val,
//         alias: alias.into(),
//     })
// }

// /// Select multiple columns from a table i.e. SELECT COL1, COL2, ...
// #[must_use]
// pub fn cols_res(names: &[&str]) -> Vec<SelectResultExpr> {
//     names.iter().map(|name| col_res(col(name), name)).collect()
// }

/// Compute the minimum of an expression and give it an alias i.e. SELECT MIN(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
// #[must_use]
// pub fn min_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
//     SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
//         expr: min(expr),
//         alias,
//     })
// }

/// Compute the maximum of an expression and give it an alias i.e. SELECT MAX(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
// #[must_use]
// pub fn max_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
//     SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
//         expr: max(expr),
//         alias: alias.into(),
//     })
// }

/// Compute the sum of an expression and give it an alias i.e. SELECT SUM(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
// #[must_use]
// pub fn sum_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
//     SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
//         expr: sum(expr),
//         alias: alias.into()
//     })
// }

/// Count the amount of non-null entries of expression and give it an alias i.e. SELECT COUNT(EXPR) AS ALIAS
///
/// # Panics
///
// /// This function will panic if the `alias` cannot be parsed.
// #[must_use]
// pub fn count_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
//     SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
//         expr: count(expr),
//         alias: alias.into(),
//     })
// }

/// Count rows and give the result an alias i.e. SELECT COUNT(*) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
// #[must_use]
// pub fn count_all_res(alias: &str) -> SelectResultExpr {
//     SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
//         expr: Expression::Aggregation {
//             op: AggregationOperator::Count,
//             expr: Box::new(Expr::Wildcard),
//         }
//         .into(),
//         alias: alias.into()
//     })
// }

/// Generate a `SetExpression` of the kind SELECT COL1, COL2, ... FROM TAB WHERE EXPR GROUP BY ...
#[must_use]
pub fn query(
    result_exprs: Vec<SelectResultExpr>,
    tab: Box<TableExpression>,
    where_expr: Box<Expression>,
    group_by: Vec<Identifier>,
) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
        result_exprs,
        from: vec![tab],
        where_expr: Some(where_expr),
        group_by,
    })
}

/// Generate a `SetExpression` of the kind SELECT COL1, COL2, ... FROM TAB GROUP BY ...
///
/// Note that there is no WHERE clause.
#[must_use]
pub fn query_all(
    result_exprs: Vec<SelectResultExpr>,
    tab: Box<TableExpression>,
    group_by: Vec<Identifier>,
) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
        result_exprs,
        from: vec![tab],
        where_expr: None,
        group_by,
    })
}

/// Generate a query of the kind SELECT ... ORDER BY ... [LIMIT ... OFFSET ...]
///
/// Note that `expr` is a boxed `SetExpression`
#[must_use]
pub fn select(
    expr: Box<SetExpression>,
    order_by: Vec<OrderBy>,
    slice: Option<Slice>,
) -> SelectStatement {
    SelectStatement {
        expr,
        order_by,
        slice,
    }
}

/// Order by one column i.e. ORDER BY ID [ASC|DESC]
///
/// # Panics
///
/// This function will panic if the `id` cannot be parsed into an identifier.
#[must_use]
pub fn order(id: &str, direction: OrderByDirection) -> Vec<OrderBy> {
    vec![OrderBy {
        expr: id.parse().unwrap(),
        direction,
    }]
}

/// Order by multiple columns i.e. ORDER BY ID0 [ASC|DESC], ID1 [ASC|DESC], ...
///
/// # Panics
///
/// This function will panic if any of the `ids` cannot be parsed
/// into an identifier.
#[must_use]
pub fn orders(ids: &[&str], directions: &[OrderByDirection]) -> Vec<OrderBy> {
    ids.iter()
        .zip(directions.iter())
        .map(|(id, dir)| OrderBy {
            expr: id.parse().unwrap(),
            direction: *dir,
        })
        .collect::<Vec<_>>()
}

/// Slice a query result using `LIMIT` and `OFFSET` clauses i.e. LIMIT N OFFSET M
#[must_use]
pub fn slice(number_rows: u64, offset_value: i64) -> Option<Slice> {
    Some(Slice {
        number_rows,
        offset_value,
    })
}

/// Group by clause with multiple columns i.e. GROUP BY ID0, ID1, ...
///
/// # Panics
///
/// This function will panic if any of the `ids` cannot be parsed
/// into an identifier.
#[must_use]
pub fn group_by(ids: &[&str]) -> Vec<Identifier> {
    ids.iter().map(|id| id.parse().unwrap()).collect()
}
