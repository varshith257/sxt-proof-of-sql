use super::{ExpressionEvaluationError, ExpressionEvaluationResult};
use crate::base::{
    database::{OwnedColumn, OwnedTable},
    math::{
        decimal::{try_convert_intermediate_decimal_to_scalar, DecimalError, Precision},
        BigDecimalExt,
    },
    scalar::Scalar,
};
use alloc::{format, string::ToString, vec};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use sqlparser::ast::{
    BinaryOperator, DataType, ExactNumberInfo, Expr, Ident, TimezoneInfo, UnaryOperator, Value,
};

impl<S: Scalar> OwnedTable<S> {
    /// Evaluate an expression on the table.
    pub fn evaluate(&self, expr: &Expr) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        match expr {
            Expr::Identifier(ident) => self.evaluate_column(ident),
            Expr::Value(_) | Expr::TypedString { .. } => self.evaluate_literal(expr),
            Expr::BinaryOp { op, left, right } => {
                self.evaluate_binary_expr(&(*op).clone().into(), left, right)
            }
            Expr::UnaryOp { op, expr } => self.evaluate_unary_expr((*op).into(), expr),
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Expression {expr:?} is not supported yet"),
            }),
        }
    }

    fn evaluate_column(&self, identifier: &Ident) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        Ok(self
            .inner_table()
            .get(identifier)
            .ok_or(ExpressionEvaluationError::ColumnNotFound {
                error: identifier.to_string(),
            })?
            .clone())
    }

    fn evaluate_literal(&self, value: &Expr) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let len = self.num_rows();
        match value {
            Expr::Value(Value::Boolean(b)) => Ok(OwnedColumn::Boolean(vec![*b; len])),
            Expr::Value(Value::Number(n, _)) => {
                let num = n
                    .parse::<i128>()
                    .map_err(|_| DecimalError::InvalidDecimal {
                        error: format!("Invalid number: {n}"),
                    })?;
                if num >= i64::MIN as i128 && num <= i64::MAX as i128 {
                    Ok(OwnedColumn::BigInt(vec![num as i64; len]))
                } else {
                    Ok(OwnedColumn::Int128(vec![num; len]))
                }
            }
            Expr::Value(Value::SingleQuotedString(s)) => {
                Ok(OwnedColumn::VarChar(vec![s.clone(); len]))
            }
            Expr::TypedString { data_type, value } => match data_type {
                DataType::Decimal(ExactNumberInfo::PrecisionAndScale(precision, scale)) => {
                    let d = value.parse::<BigDecimalExt>().map_err(|_| {
                        DecimalError::InvalidDecimal {
                            error: format!("Invalid decimal value: {value}"),
                        }
                    })?;
                    let raw_scale = d.scale();
                    let scale = raw_scale
                        .try_into()
                        .map_err(|_| DecimalError::InvalidScale {
                            scale: raw_scale.to_string(),
                        })?;
                    let precision = Precision::try_from(d.precision())?;
                    let scalar = try_convert_intermediate_decimal_to_scalar(&d, precision, scale)?;
                    Ok(OwnedColumn::Decimal75(precision, scale, vec![scalar; len]))
                }
                DataType::Timestamp(Some(time_unit), Some(time_zone)) => {
                    let time_unit = PoSQLTimeUnit::try_from(*time_unit).map_err(|err| {
                        DecimalError::InvalidDecimal {
                            error: format!("Invalid time unit precision: {err}"),
                        }
                    })?;

                    let timezone_info = time_zone.clone();

                    let timestamp_value =
                        value
                            .parse::<i64>()
                            .map_err(|_| DecimalError::InvalidDecimal {
                                error: format!("Invalid timestamp value: {value}"),
                            })?;
                    Ok(OwnedColumn::TimestampTZ(
                        time_unit,
                        // Default to UTC
                        timezone_info,
                        vec![timestamp_value; len],
                    ))
                }
                DataType::Timestamp(Some(time_unit), None) => {
                    let time_unit = PoSQLTimeUnit::try_from(*time_unit).map_err(|err| {
                        DecimalError::InvalidDecimal {
                            error: format!("Invalid time unit precision: {err}"),
                        }
                    })?;

                    // Default to UTC if no timezone is provided
                    let timestamp_value =
                        value
                            .parse::<i64>()
                            .map_err(|_| DecimalError::InvalidDecimal {
                                error: format!("Invalid timestamp value: {value}"),
                            })?;

                    Ok(OwnedColumn::TimestampTZ(
                        time_unit,
                        // Default to UTC
                        TimezoneInfo::None,
                        vec![timestamp_value; len],
                    ))
                }
            },
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: "Unsupported expression type".to_string(),
            }),
        }
    }

    // fn evaluate_literal(&self, value: &Value) -> ExpressionEvaluationResult<OwnedColumn<S>> {
    //     let len = self.num_rows();
    //     match value {
    //         Value::Boolean(b) => Ok(OwnedColumn::Boolean(vec![*b; len])),
    //         Value::Number(num, _) => {
    //             let bigint_value = num_bigint::BigInt::parse_bytes(num.as_bytes(), 10)
    //                 .ok_or(DecimalError::InvalidDecimal {
    //                     error: format!("Failed to parse '{num}' as a decimal"),
    //                 })?;
    //             Ok(OwnedColumn::BigInt(vec![bigint_value.to_i64().unwrap(); len]))
    //         }
    //         Value::SingleQuotedString(s) => Ok(OwnedColumn::VarChar(vec![s.clone(); len])),
    //         Value::Timestamp { value, .. } => {
    //             let timestamp = value.parse::<i64>().map_err(|_| {
    //                 ExpressionEvaluationError::Unsupported {
    //                     expression: format!("Invalid timestamp value: {value}"),
    //                 }
    //             })?;
    //             Ok(OwnedColumn::TimestampTZ(
    //                 PoSQLTimeUnit::Second,
    //                 PoSQLTimeZone::Utc,
    //                 vec![timestamp; len],
    //             ))
    //         }
    //         _ => Err(ExpressionEvaluationError::Unsupported {
    //             expression: format!("Literal value {value:?} is not supported yet"),
    //         }),
    //     }
    // }
    fn evaluate_unary_expr(
        &self,
        op: UnaryOperator,
        expr: &Expr,
    ) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let column = self.evaluate(expr)?;
        match op {
            UnaryOperator::Not => Ok(column.element_wise_not()?),
            // Handle unsupported unary operators
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Unary operator '{op}' is not supported."),
            }),
        }
    }

    fn evaluate_binary_expr(
        &self,
        op: &BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match op {
            BinaryOperator::And => Ok(left.element_wise_and(&right)?),
            BinaryOperator::Or => Ok(left.element_wise_or(&right)?),
            BinaryOperator::Eq => Ok(left.element_wise_eq(&right)?),
            BinaryOperator::GtEq => Ok(left.element_wise_ge(&right)?),
            BinaryOperator::LtEq => Ok(left.element_wise_le(&right)?),
            BinaryOperator::Plus => Ok(left.element_wise_add(&right)?),
            BinaryOperator::Minus => Ok(left.element_wise_sub(&right)?),
            BinaryOperator::Multiply => Ok(left.element_wise_mul(&right)?),
            BinaryOperator::Divide => Ok(left.element_wise_div(&right)?),
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Binary operator '{op}' is not supported."),
            }),
        }
    }
}
