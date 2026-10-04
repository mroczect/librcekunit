//! CSV validation helpers for the replace-all workflow.
//!
//! Pure, no I/O. Given the raw text of a CSV file, these helpers
//! return a summary or a structured error. No network, no disk.
//!
//! The parser is a state machine that reads the whole document, not
//! one line at a time. This is required because CSV fields may contain
//! embedded newlines when they are wrapped in double quotes.

/// Result of validating a CSV file.
#[derive(Debug, Clone)]
pub struct CsvSummary {
    /// Header fields exactly as they appeared in the first record.
    pub header: Vec<String>,
    /// Number of data records (excludes the header).
    pub data_rows: usize,
    /// Index of `no_perjanjian` in the header, if present.
    pub no_perjanjian_index: Option<usize>,
    /// First three data records, for preview.
    pub preview_rows: Vec<Vec<String>>,
}

/// Error returned by CSV validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CsvError {
    /// The file was empty or contained only whitespace.
    Empty,
    /// The header record was missing.
    NoHeader,
    /// A data record had a different number of fields than the header.
    FieldCountMismatch {
        /// 1-based record index in the original file.
        record: usize,
        /// Fields in the header.
        expected: usize,
        /// Fields in the offending record.
        actual: usize,
    },
    /// The required column `no_perjanjian` is missing from the header.
    MissingRequiredColumn(String),
    /// A quoted field was not closed before the end of the file.
    UnterminatedQuote {
        /// 1-based record index.
        record: usize,
    },
}

impl core::fmt::Display for CsvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "file is empty"),
            Self::NoHeader => write!(f, "file has no header record"),
            Self::FieldCountMismatch {
                record,
                expected,
                actual,
            } => write!(
                f,
                "record {record}: expected {expected} fields, found {actual}"
            ),
            Self::MissingRequiredColumn(name) => {
                write!(f, "missing required column: {name}")
            }
            Self::UnterminatedQuote { record } => {
                write!(f, "record {record}: unterminated quoted field")
            }
        }
    }
}

impl core::error::Error for CsvError {}

/// Columns that must be present in the header for the upload to be
/// accepted by the server.
pub const REQUIRED_COLUMNS: &[&str] = &["no_perjanjian"];

/// Maximum number of fields in a single record that the validator
/// will accept. Guards against a runaway parser on malformed input.
const MAX_FIELDS: usize = 1024;

/// Validate CSV text.
///
/// # Errors
///
/// Returns a [`CsvError`] variant describing the first problem found.
pub fn validate(text: &str) -> Result<CsvSummary, CsvError> {
    let records = parse_records(text)?;

    let mut iter = records.into_iter().enumerate();

    let (_, header) = iter.next().ok_or(CsvError::Empty)?;

    if header.is_empty() || (header.len() == 1 && header.first().is_some_and(String::is_empty)) {
        return Err(CsvError::NoHeader);
    }

    for required in REQUIRED_COLUMNS {
        if !header.iter().any(|h| h == required) {
            return Err(CsvError::MissingRequiredColumn((*required).to_owned()));
        }
    }

    let no_perjanjian_index = header.iter().position(|h| h == "no_perjanjian");
    let expected_fields = header.len();

    let mut data_rows: usize = 0;
    let mut preview_rows: Vec<Vec<String>> = Vec::new();

    for (idx, record) in iter {
        let record_number = idx.saturating_add(1);
        // Skip completely empty trailing records.
        if record.len() == 1 && record.first().is_some_and(String::is_empty) {
            continue;
        }
        if record.len() != expected_fields {
            return Err(CsvError::FieldCountMismatch {
                record: record_number,
                expected: expected_fields,
                actual: record.len(),
            });
        }
        data_rows = data_rows.saturating_add(1);
        if preview_rows.len() < 3 {
            preview_rows.push(record);
        }
    }

    if data_rows == 0 {
        return Err(CsvError::Empty);
    }

    Ok(CsvSummary {
        header,
        data_rows,
        no_perjanjian_index,
        preview_rows,
    })
}

/// Parse the entire CSV text into records.
///
/// Handles double-quoted fields with embedded commas and newlines.
/// Doubled quotes inside a quoted field (`""`) are unescaped to a
/// single `"`.
///
/// # Errors
///
/// Returns [`CsvError::UnterminatedQuote`] when a quoted field is not
/// closed before the end of the input.
pub fn parse_records(text: &str) -> Result<Vec<Vec<String>>, CsvError> {
    // Strip UTF-8 BOM if present, so the first header column is
    // `no` rather than `\u{FEFF}no`.
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);

    let mut records: Vec<Vec<String>> = Vec::new();
    let mut current_record: Vec<String> = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut record_index: usize = 1;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        current_field.push('"');
                        let _ = chars.next();
                    } else {
                        in_quotes = false;
                    }
                }
                other => current_field.push(other),
            }
        } else {
            match c {
                '"' if current_field.is_empty() => in_quotes = true,
                ',' => {
                    if current_record.len() >= MAX_FIELDS {
                        return Err(CsvError::FieldCountMismatch {
                            record: record_index,
                            expected: MAX_FIELDS,
                            actual: current_record.len().saturating_add(1),
                        });
                    }
                    current_record.push(core::mem::take(&mut current_field));
                }
                '\r' => {
                    // Ignore carriage returns outside quotes. The
                    // matching newline will terminate the record.
                }
                '\n' => {
                    current_record.push(core::mem::take(&mut current_field));
                    records.push(core::mem::take(&mut current_record));
                    record_index = record_index.saturating_add(1);
                }
                other => current_field.push(other),
            }
        }
    }

    if in_quotes {
        return Err(CsvError::UnterminatedQuote {
            record: record_index,
        });
    }

    // Flush the final field and record if the file does not end with
    // a newline.
    if !current_field.is_empty() || !current_record.is_empty() {
        current_record.push(current_field);
        records.push(current_record);
    }

    Ok(records)
}
