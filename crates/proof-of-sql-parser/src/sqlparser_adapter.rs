use crate::{
    intermediate_ast::{OrderBy as IntermediateOrderBy, OrderByDirection, Slice},
    Identifier,
};
use alloc::{string::ToString, vec};
use sqlparser::ast::{Expr, Offset, OffsetRows, OrderByExpr, Value};

/// Converts an [`IntermediateOrderBy`] from the intermediate AST to a [`OrderByExpr`] for the SQL parser.
impl From<IntermediateOrderBy> for OrderByExpr {
    fn from(intermediate_order_by: IntermediateOrderBy) -> Self {
        // Convert Identifier to Expr
        let expr = Expr::Identifier(intermediate_order_by.expr.into());

        let asc = match intermediate_order_by.direction {
            OrderByDirection::Asc => Some(true),
            OrderByDirection::Desc => Some(false),
        };

        // Create the OrderByExpr
        OrderByExpr {
            expr,
            asc,
            nulls_first: None,
        }
    }
}

/// Converts a [`Slice`] representing pagination into an [`Offset`] for the SQL parser.
impl From<Slice> for Offset {
    fn from(slice: Slice) -> Self {
        let value_expr = Expr::Value(Value::Number(slice.offset_value.to_string(), false)); // Convert offset_value to an Expr

        let rows = match slice.number_rows {
            u64::MAX => OffsetRows::None, // No specific row offset
            1 => OffsetRows::Row,         // For a single row offset
            _ => OffsetRows::Rows,        // For multiple rows
        };

        Offset {
            value: value_expr,
            rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intermediate_ast::{OrderBy, OrderByDirection};
    use sqlparser::ast::{Expr, Ident, Offset, OffsetRows, OrderByExpr, Value};

    #[test]
    fn test_conversion_order_by_asc() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new("column_name"),
            direction: OrderByDirection::Asc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.asc, Some(true));
        assert_eq!(
            order_by_expr.expr,
            Expr::Identifier(Ident::new("column_name"))
        );
    }

    #[test]
    fn test_conversion_order_by_desc() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new("column_name"),
            direction: OrderByDirection::Desc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.asc, Some(false));
        assert_eq!(
            order_by_expr.expr,
            Expr::Identifier(Ident::new("column_name"))
        );
    }

    #[test]
    fn test_conversion_order_by_nulls_first() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new("column_name"),
            direction: OrderByDirection::Asc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.nulls_first, None);
    }

    #[test]
    fn test_edge_case_empty_order_by() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new(""),
            direction: OrderByDirection::Asc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.asc, Some(true));
        assert_eq!(order_by_expr.expr, Expr::Identifier(Ident::new("")));
    }

    #[test]
    fn test_slice_to_offset_conversion_all_rows() {
        let slice = Slice {
            number_rows: u64::MAX,
            offset_value: 0,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("0".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::None);
    }

    #[test]
    fn test_slice_to_offset_conversion_single_row() {
        let slice = Slice {
            number_rows: 1,
            offset_value: 5,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("5".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::Row);
    }

    #[test]
    fn test_slice_to_offset_conversion_multiple_rows() {
        let slice = Slice {
            number_rows: 10,
            offset_value: -2,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("-2".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::Rows);
    }

    #[test]
    fn test_slice_to_offset_conversion_zero_offset() {
        let slice = Slice {
            number_rows: 10,
            offset_value: 0,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("0".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::Rows);
    }
}
