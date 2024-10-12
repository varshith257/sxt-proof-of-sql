use super::intermediate_ast::{OrderBy, SetExpression, Slice, TableExpression};
use crate::{sql::SelectStatementParser, Identifier, ParseError, ParseResult, ResourceId};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use core::{fmt, str::FromStr};
use serde::{Deserialize, Serialize};

/// Representation of a select statement, that is, the only type of queries allowed.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SelectStatement {
    /// the query expression
    pub expr: Box<SetExpression>,

    /// if non-empty, an sort-order that is applied to the rows returned as result
    pub order_by: Vec<OrderBy>,

    /// an optional slice clause, which can restrict the rows returned to a window within the
    /// set of rows as generated by `expr` and `order_by`.
    pub slice: Option<Slice>,
}

impl fmt::Debug for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SelectStatement \n[{:#?},\n{:#?},\n{:#?}\n]",
            self.expr, self.order_by, self.slice
        )
    }
}

impl SelectStatement {
    /// This function returns the referenced tables in the provided `intermediate_ast`
    ///
    /// Note that we provide a `default_schema` in case the table expression
    /// does not have any associated schema. This `default_schema` is
    /// used to construct the `resource_id`, as we cannot have this field empty.
    /// In case the table expression already has an associated schema,
    /// then it's used instead of `default_schema`. Although the DQL endpoint
    /// would require both to be equal, we have chosen to not fail here
    /// as this would imply the caller to always know beforehand the referenced
    /// schemas.
    ///
    /// Return:
    /// - The vector with all tables referenced by the intermediate ast, encoded as resource ids.
    #[must_use]
    pub fn get_table_references(&self, default_schema: Identifier) -> Vec<ResourceId> {
        let set_expression: &SetExpression = &(self.expr);

        match set_expression {
            SetExpression::Query {
                result_exprs: _,
                from,
                where_expr: _,
                group_by: _,
            } => convert_table_expr_to_resource_id_vector(&from[..], default_schema),
        }
    }
}

impl FromStr for SelectStatement {
    type Err = crate::ParseError;

    fn from_str(query: &str) -> ParseResult<Self> {
        SelectStatementParser::new()
            .parse(query)
            .map_err(|e| ParseError::QueryParseError {
                error: e.to_string(),
            })
    }
}

/// # Panics
///
/// This function will panic in the following cases:
/// - If `ResourceId::try_new` fails to create a valid `ResourceId`,
///   the `.unwrap()` call will cause a panic.
fn convert_table_expr_to_resource_id_vector(
    table_expressions: &[Box<TableExpression>],
    default_schema: Identifier,
) -> Vec<ResourceId> {
    let mut tables = Vec::new();

    for table_expression in table_expressions {
        let table_ref: &TableExpression = table_expression;

        match table_ref {
            TableExpression::Named { table, schema } => {
                let schema = schema.as_ref().map_or_else(
                    || default_schema.name(),
                    super::identifier::Identifier::as_str,
                );

                tables.push(ResourceId::try_new(schema, table.as_str()).unwrap());
            }
        }
    }

    tables
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::SelectStatementParser;

    #[test]
    fn we_can_get_the_correct_table_references_using_a_default_schema() {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("SELECT A FROM TAB WHERE C = 3")
            .unwrap();
        let default_schema = Identifier::try_new("ETH").unwrap();
        let ref_tables = parsed_query_ast.get_table_references(default_schema);

        // note: the parsed table is always lower case
        assert_eq!(ref_tables, [ResourceId::try_new("eth", "tab").unwrap()]);
    }

    #[test]
    fn we_can_get_the_correct_table_references_in_case_the_default_schema_equals_the_original_schema(
    ) {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("SELECT A FROM SCHEMA.TAB WHERE C = 3")
            .unwrap();
        let default_schema = Identifier::try_new("SCHEMA").unwrap();
        let ref_tables = parsed_query_ast.get_table_references(default_schema);

        assert_eq!(ref_tables, [ResourceId::try_new("schema", "tab").unwrap()]);
    }

    #[test]
    fn we_can_get_the_correct_table_references_in_case_the_default_schema_differs_from_the_original_schema(
    ) {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("SELECT A FROM SCHEMA.TAB WHERE C = 3")
            .unwrap();
        let default_schema = Identifier::try_new("  ETH  ").unwrap();
        let ref_tables = parsed_query_ast.get_table_references(default_schema);

        assert_eq!(ref_tables, [ResourceId::try_new("schema", "tab").unwrap()]);
    }
}
