use alloc::sync::Arc;
use arrow::array::{
    TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
    TimestampSecondArray,
};
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;

/// Extension trait for `Vec<T>` to convert it to an Arrow array
pub trait ToArrow {
    /// Returns the equivalent Arrow type
    fn to_type(&self) -> arrow::datatypes::DataType;
    /// Converts the `Vec<T>` to an Arrow `ArrayRef`.
    fn to_array(self) -> Arc<dyn arrow::array::Array>;
}

impl ToArrow for Vec<bool> {
    fn to_type(&self) -> arrow::datatypes::DataType {
        arrow::datatypes::DataType::Boolean
    }

    fn to_array(self) -> Arc<dyn arrow::array::Array> {
        Arc::new(<arrow::array::BooleanArray>::from(self))
    }
}

/// A wrapper around i64 to mitigate conflicting From<i64>
/// implementations
#[derive(Clone)]
pub struct Time {
    /// i64 count of timeunits since unix epoch
    pub timestamp: i64,
    /// Timeunit of this time
    pub unit: PoSQLTimeUnit,
}

impl ToArrow for Vec<Time> {
    fn to_type(&self) -> arrow::datatypes::DataType {
        match self.first().map(|time| time.unit) {
            Some(PoSQLTimeUnit::Second) => {
                arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Second, None)
            }
            Some(PoSQLTimeUnit::Millisecond) => {
                arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None)
            }
            Some(PoSQLTimeUnit::Microsecond) => {
                arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None)
            }
            Some(PoSQLTimeUnit::Nanosecond) => {
                arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, None)
            }
            None => panic!("Empty Vec<Time> cannot determine TimeUnit"),
        }
    }

    fn to_array(self) -> Arc<dyn arrow::array::Array> {
        match self.first().map(|time| time.unit) {
            Some(PoSQLTimeUnit::Second) => {
                let raw_data: Vec<i64> = self.into_iter().map(|time| time.timestamp).collect();
                Arc::new(TimestampSecondArray::from(raw_data))
            }
            Some(PoSQLTimeUnit::Millisecond) => {
                let raw_data: Vec<i64> = self.into_iter().map(|time| time.timestamp).collect();
                Arc::new(TimestampMillisecondArray::from(raw_data))
            }
            Some(PoSQLTimeUnit::Microsecond) => {
                let raw_data: Vec<i64> = self.into_iter().map(|time| time.timestamp).collect();
                Arc::new(TimestampMicrosecondArray::from(raw_data))
            }
            Some(PoSQLTimeUnit::Nanosecond) => {
                let raw_data: Vec<i64> = self.into_iter().map(|time| time.timestamp).collect();
                Arc::new(TimestampNanosecondArray::from(raw_data))
            }
            None => panic!("Empty Vec<Time> cannot determine TimeUnit"),
        }
    }
}

/// TODO: add docs
macro_rules! int_to_arrow_array {
    ($tt:ty, $dtt:expr, $att:ty) => {
        impl ToArrow for Vec<$tt> {
            fn to_type(&self) -> arrow::datatypes::DataType {
                $dtt
            }

            fn to_array(self) -> Arc<dyn arrow::array::Array> {
                // this cast normalizes the table as we only support i64 values
                let v = self.iter().map(|v| *v).collect::<Vec<_>>();
                Arc::new(<$att>::from(v))
            }
        }
    };
}

int_to_arrow_array!(
    i16,
    arrow::datatypes::DataType::Int16,
    arrow::array::Int16Array
);

int_to_arrow_array!(
    i32,
    arrow::datatypes::DataType::Int32,
    arrow::array::Int32Array
);

int_to_arrow_array!(
    i64,
    arrow::datatypes::DataType::Int64,
    arrow::array::Int64Array
);

impl ToArrow for Vec<i128> {
    fn to_type(&self) -> arrow::datatypes::DataType {
        arrow::datatypes::DataType::Decimal128(38, 0)
    }

    ///
    /// # Panics
    ///
    /// Will panic if the conversion to a [`Decimal128Array`](arrow::array::Decimal128Array) fails, which can happen if the data exceeds the specified precision and scale (38, 0). Ensure that all values are within the valid range for the Decimal128 type.
    fn to_array(self) -> Arc<dyn arrow::array::Array> {
        Arc::new(
            arrow::array::Decimal128Array::from(self)
                .with_precision_and_scale(38, 0)
                .unwrap(),
        )
    }
}

/// TODO: add docs
macro_rules! string_to_arrow_array {
    ($tt:ty, $dtt:expr, $att:ty) => {
        impl ToArrow for Vec<$tt> {
            fn to_type(&self) -> arrow::datatypes::DataType {
                $dtt
            }

            fn to_array(self) -> Arc<dyn arrow::array::Array> {
                Arc::new(<$att>::from(self))
            }
        }
    };
}

string_to_arrow_array!(
    &str,
    arrow::datatypes::DataType::Utf8,
    arrow::array::StringArray
);
string_to_arrow_array!(
    String,
    arrow::datatypes::DataType::Utf8,
    arrow::array::StringArray
);

/// Utility macro to simplify the creation of [`RecordBatch`](arrow::record_batch::RecordBatch) instances
#[macro_export]
///
/// # Panics
///
/// Will panic if the `RecordBatch` creation fails. This can occur if:
/// - The lengths of the provided slices are not equal.
/// - The `to_array()` method on any slice returns an error, indicating invalid data types or mismatched lengths.
macro_rules! record_batch {
    ($($col_name:expr => $slice:expr), + $(,)?) => {
        {
            use alloc::sync::Arc;
            use arrow::datatypes::Field;
            use arrow::datatypes::Schema;
            use arrow::record_batch::RecordBatch;
            use $crate::base::database::ToArrow;

            let schema = Arc::new(Schema::new(
                vec![$(
                    Field::new(&$col_name.to_string(), $slice.to_vec().to_type(), false)
                ,)+]));

            let arrays = vec![$($slice.to_vec().to_array(),)+];
            RecordBatch::try_new(schema, arrays).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::record_batch;
    use alloc::sync::Arc;
    use arrow::{
        datatypes::{Field, Schema},
        record_batch::RecordBatch,
    };

    #[test]
    fn test_record_batch_macro() {
        let batch = record_batch!(
            "f" => ["abc", "t", "fg"],
            "ghisi" => [-99_i64, 1230, 222],
            "boolean" => [true, false, true],
        );

        let arrays: Vec<Arc<dyn arrow::array::Array>> = vec![
            Arc::new(arrow::array::StringArray::from(["abc", "t", "fg"].to_vec())),
            Arc::new(arrow::array::Int64Array::from(
                [-99_i64, 1230, 222].to_vec(),
            )),
            Arc::new(arrow::array::BooleanArray::from(
                [true, false, true].to_vec(),
            )),
        ];

        let schema = Arc::new(Schema::new(vec![
            Field::new("f", arrow::datatypes::DataType::Utf8, false),
            Field::new("ghisi", arrow::datatypes::DataType::Int64, false),
            Field::new("boolean", arrow::datatypes::DataType::Boolean, false),
        ]));

        let expected_batch = RecordBatch::try_new(schema, arrays).unwrap();

        assert_eq!(batch, expected_batch);
    }

    #[test]
    fn we_can_create_a_record_batch_with_i128_values() {
        let batch = record_batch!(
            "ghisi" => [-99_i128, 1230, 222, i128::MAX, i128::MIN]
        );

        let arrays: Vec<Arc<dyn arrow::array::Array>> = vec![Arc::new(
            arrow::array::Decimal128Array::from(
                [-99_i128, 1230, 222, i128::MAX, i128::MIN].to_vec(),
            )
            .with_precision_and_scale(38, 0)
            .unwrap(),
        )];

        let schema = Arc::new(Schema::new(vec![Field::new(
            "ghisi",
            arrow::datatypes::DataType::Decimal128(38, 0),
            false,
        )]));

        let expected_batch = RecordBatch::try_new(schema, arrays).unwrap();

        assert_eq!(batch, expected_batch);
    }
}
