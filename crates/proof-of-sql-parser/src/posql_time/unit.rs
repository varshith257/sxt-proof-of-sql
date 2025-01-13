use super::PoSQLTimestampError;
use core::fmt;
use serde::{Deserialize, Serialize};

/// An intermediate type representing the time units from a parsed query
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoSQLTimeUnit {
    /// Defaults and Represents nanoseconds with precision 9: ex "2024-06-20 12:34:56.123456789"
    Nanosecond,
}

impl From<PoSQLTimeUnit> for u64 {
    fn from(value: PoSQLTimeUnit) -> u64 {
        match value {
            PoSQLTimeUnit::Nanosecond => 9,
        }
    }
}

impl TryFrom<&str> for PoSQLTimeUnit {
    type Error = PoSQLTimestampError;
    fn try_from(value: &str) -> Result<Self, PoSQLTimestampError> {
        match value {
            "9" => Ok(PoSQLTimeUnit::Nanosecond),
            _ => Err(PoSQLTimestampError::UnsupportedPrecision {
                error: value.into(),
            }),
        }
    }
}

impl fmt::Display for PoSQLTimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoSQLTimeUnit::Nanosecond => write!(f, "nanoseconds (precision: 9)"),
        }
    }
}

// allow(deprecated) for the sole purpose of testing that
// timestamp precision is parsed correctly.
#[cfg(test)]
#[allow(deprecated, clippy::missing_panics_doc)]
mod time_unit_tests {
    use super::*;
    use crate::posql_time::{PoSQLTimestamp, PoSQLTimestampError};
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_valid_precisions() {
        assert_eq!(PoSQLTimeUnit::try_from("9"), Ok(PoSQLTimeUnit::Nanosecond));
    }

    #[test]
    fn test_invalid_precision() {
        let invalid_precisions = [
            "1", "2", "4", "5", "7", "8", "10", "zero", "three", "cat", "-1", "-2",
        ]; // Testing all your various invalid inputs
        for &value in &invalid_precisions {
            let result = PoSQLTimeUnit::try_from(value);
            assert!(matches!(
                result,
                Err(PoSQLTimestampError::UnsupportedPrecision { .. })
            ));
        }
    }
    #[test]
    fn test_rfc3339_timestamp_with_nanoseconds() {
        let input = "2023-06-26T12:34:56.123456789Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_nano(12, 34, 56, 123_456_789);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit(), PoSQLTimeUnit::Nanosecond);
        assert_eq!(
            result.timestamp().timestamp_nanos_opt().unwrap(),
            expected.timestamp_nanos_opt().unwrap()
        );
    }
}
