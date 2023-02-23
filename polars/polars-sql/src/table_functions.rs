use std::str::FromStr;

use polars_core::prelude::{PolarsError, PolarsResult};
#[cfg(feature = "csv")]
use polars_lazy::prelude::LazyCsvReader;
use polars_lazy::prelude::LazyFrame;
use sqlparser::ast::{FunctionArg, FunctionArgExpr};

/// Table functions that are supported by Polars
pub(crate) enum PolarsTableFunctions {
    /// SQL 'read_csv' function
    /// ```sql
    /// SELECT * FROM read_csv('path/to/file.csv')
    /// ```
    #[cfg(feature = "csv")]
    ReadCsv,
    /// SQL 'read_parquet' function
    /// ```sql
    /// SELECT * FROM read_parquet('path/to/file.parquet')
    /// ```
    #[cfg(feature = "parquet")]
    ReadParquet,
    /// SQL 'read_ipc' function
    /// ```sql
    /// SELECT * FROM read_ipc('path/to/file.ipc')
    /// ```
    #[cfg(feature = "ipc")]
    ReadIpc,
}

impl FromStr for PolarsTableFunctions {
    type Err = PolarsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "csv")]
            "read_csv" => Ok(PolarsTableFunctions::ReadCsv),
            #[cfg(feature = "parquet")]
            "read_parquet" => Ok(PolarsTableFunctions::ReadParquet),
            #[cfg(feature = "ipc")]
            "read_ipc" => Ok(PolarsTableFunctions::ReadIpc),
            _ => Err(PolarsError::ComputeError(
                format!("'{}' is not a supported table function", s).into(),
            )),
        }
    }
}

impl PolarsTableFunctions {
    pub(crate) fn execute(&self, args: &[FunctionArg]) -> PolarsResult<(String, LazyFrame)> {
        match self {
            #[cfg(feature = "csv")]
            PolarsTableFunctions::ReadCsv => self.read_csv(args),
            #[cfg(feature = "parquet")]
            PolarsTableFunctions::ReadParquet => self.read_parquet(args),
            #[cfg(feature = "ipc")]
            PolarsTableFunctions::ReadIpc => self.read_ipc(args),
            _ => unreachable!(),
        }
    }

    #[cfg(feature = "csv")]
    fn read_csv(&self, args: &[FunctionArg]) -> PolarsResult<(String, LazyFrame)> {
        use polars_lazy::frame::LazyFileListReader;
        let path = self.get_file_path_from_arg(&args[0])?;
        let lf = LazyCsvReader::new(&path).finish()?;
        Ok((path, lf))
    }

    #[cfg(feature = "parquet")]
    fn read_parquet(&self, args: &[FunctionArg]) -> PolarsResult<(String, LazyFrame)> {
        let path = self.get_file_path_from_arg(&args[0])?;
        let lf = LazyFrame::scan_parquet(&path, Default::default())?;
        Ok((path, lf))
    }

    #[cfg(feature = "ipc")]
    fn read_ipc(&self, args: &[FunctionArg]) -> PolarsResult<(String, LazyFrame)> {
        let path = self.get_file_path_from_arg(&args[0])?;
        let lf = LazyFrame::scan_ipc(&path, Default::default())?;
        Ok((path, lf))
    }

    fn get_file_path_from_arg(&self, arg: &FunctionArg) -> PolarsResult<String> {
        use sqlparser::ast::{Expr as SqlExpr, Value as SqlValue};
        match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(SqlExpr::Value(
                SqlValue::SingleQuotedString(s),
            ))) => Ok(s.to_string()),
            _ => Err(PolarsError::ComputeError(
                format!("Only a single quoted string is accepted as the first parameter. Instead received: {}", arg).into(),
            )),
        }
    }
}