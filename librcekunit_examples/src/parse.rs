use scraper::{ElementRef, Html, Selector};

/// One row parsed from the Cek Unit dashboard table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CekUnitRow {
    /// Column: No.
    pub no: String,
    /// Column: No Perjanjian.
    pub no_perjanjian: String,
    /// Column: Nama Nasabah.
    pub nama_nasabah: String,
    /// Column: Nopol.
    pub nopol: String,
    /// Column: Coll.
    pub coll: String,
    /// Column: PIC.
    pub pic: String,
    /// Column: Kategori.
    pub kategori: String,
    /// Column: JTO.
    pub jto: String,
    /// Column: No Rangka.
    pub no_rangka: String,
    /// Column: No Mesin.
    pub no_mesin: String,
    /// Column: Merk.
    pub merk: String,
    /// Column: Type.
    pub r#type: String,
    /// Column: Warna.
    pub warna: String,
    /// Column: Status.
    pub status: String,
    /// Column: Actual Penyelesaian.
    pub actual_penyelesaian: String,
    /// Column: Angsuran Ke.
    pub angsuran_ke: String,
    /// Column: Tenor.
    pub tenor: String,
}

/// One row parsed from the input-user table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InputUserRow {
    /// Column: No.
    pub no: String,
    /// Column: Created at.
    pub created_at: String,
    /// Column: User ID.
    pub user_id: String,
    /// Column: Nopol.
    pub nopol: String,
    /// Column: Lokasi.
    pub lokasi: String,
    /// Column: ForN.
    pub forn: String,
    /// Column: Nama.
    pub nama: String,
    /// Column: Kategori.
    pub kategori: String,
    /// Column: Nama Nasabah.
    pub nama_nasabah: String,
    /// Column: No Perjanjian.
    pub no_perjanjian: String,
}

/// Error returned by parse helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// A CSS selector failed to compile. Indicates a programming bug.
    BadSelector(String),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BadSelector(msg) => write!(f, "bad selector: {msg}"),
        }
    }
}

impl core::error::Error for ParseError {}

/// Parse all data rows from a dashboard HTML page.
///
/// # Errors
///
/// Returns [`ParseError::BadSelector`] if an internal selector fails.
pub fn parse_dashboard_rows(html: &str) -> Result<Vec<CekUnitRow>, ParseError> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table#cekunit-table tbody tr")
        .map_err(|e| ParseError::BadSelector(format!("row: {e}")))?;
    let cell_selector =
        Selector::parse("td").map_err(|e| ParseError::BadSelector(format!("cell: {e}")))?;

    let mut rows = Vec::new();
    for row in document.select(&row_selector) {
        let cells: Vec<String> = row.select(&cell_selector).map(|c| cell_text(&c)).collect();

        if cells.len() < 17 {
            continue;
        }

        rows.push(CekUnitRow {
            no: cell_at(&cells, 0),
            no_perjanjian: cell_at(&cells, 1),
            nama_nasabah: cell_at(&cells, 2),
            nopol: cell_at(&cells, 3),
            coll: cell_at(&cells, 4),
            pic: cell_at(&cells, 5),
            kategori: cell_at(&cells, 6),
            jto: cell_at(&cells, 7),
            no_rangka: cell_at(&cells, 8),
            no_mesin: cell_at(&cells, 9),
            merk: cell_at(&cells, 10),
            r#type: cell_at(&cells, 11),
            warna: cell_at(&cells, 12),
            status: cell_at(&cells, 13),
            actual_penyelesaian: cell_at(&cells, 14),
            angsuran_ke: cell_at(&cells, 15),
            tenor: cell_at(&cells, 16),
        });
    }

    Ok(rows)
}

/// Parse all data rows from the input-user HTML page.
///
/// # Errors
///
/// Returns [`ParseError::BadSelector`] if an internal selector fails.
pub fn parse_input_user_rows(html: &str) -> Result<Vec<InputUserRow>, ParseError> {
    let document = Html::parse_document(html);
    let row_selector = Selector::parse("table#input-user-table tbody tr")
        .map_err(|e| ParseError::BadSelector(format!("row: {e}")))?;
    let cell_selector =
        Selector::parse("td").map_err(|e| ParseError::BadSelector(format!("cell: {e}")))?;

    let mut rows = Vec::new();
    for row in document.select(&row_selector) {
        let cells: Vec<String> = row.select(&cell_selector).map(|c| cell_text(&c)).collect();

        if cells.len() < 10 {
            continue;
        }

        rows.push(InputUserRow {
            no: cell_at(&cells, 0),
            created_at: cell_at(&cells, 1),
            user_id: cell_at(&cells, 2),
            nopol: cell_at(&cells, 3),
            lokasi: cell_at(&cells, 4),
            forn: cell_at(&cells, 5),
            nama: cell_at(&cells, 6),
            kategori: cell_at(&cells, 7),
            nama_nasabah: cell_at(&cells, 8),
            no_perjanjian: cell_at(&cells, 9),
        });
    }

    Ok(rows)
}

/// Parse the total record count from the pagination footer.
#[must_use]
pub fn parse_total_records(html: &str) -> Option<usize> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("nav div").ok()?;

    for element in document.select(&selector) {
        let text = element.text().collect::<String>();
        if let Some(total) = extract_total_from_text(&text) {
            return Some(total);
        }
    }
    None
}

/// Extract the total number from a phrase like `dari 7527 data`.
fn extract_total_from_text(text: &str) -> Option<usize> {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let after = normalized.split_once(" dari ")?.1;
    let first = after.split_whitespace().next()?;
    let digits: String = first.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<usize>().ok()
}

/// Return trimmed text of a single element.
fn cell_text(cell: &ElementRef<'_>) -> String {
    cell.text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Get a cell by index or return an empty string.
fn cell_at(cells: &[String], index: usize) -> String {
    cells.get(index).cloned().unwrap_or_default()
}

/// Group CekUnit rows by a caller-supplied key function.
#[must_use]
pub fn count_by<F>(rows: &[CekUnitRow], key: F) -> Vec<(String, usize)>
where
    F: Fn(&CekUnitRow) -> String,
{
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for row in rows {
        let label = key(row);
        let label = if label.is_empty() {
            String::from("<empty>")
        } else {
            label
        };
        let entry = counts.entry(label).or_insert(0);
        *entry = entry.saturating_add(1);
    }

    let mut out: Vec<(String, usize)> = counts.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// Group InputUser rows by a caller-supplied key function.
#[must_use]
pub fn count_by_input_user<F>(rows: &[InputUserRow], key: F) -> Vec<(String, usize)>
where
    F: Fn(&InputUserRow) -> String,
{
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for row in rows {
        let label = key(row);
        let label = if label.is_empty() {
            String::from("<empty>")
        } else {
            label
        };
        let entry = counts.entry(label).or_insert(0);
        *entry = entry.saturating_add(1);
    }

    let mut out: Vec<(String, usize)> = counts.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// Serialize CekUnit rows to CSV text with a fixed header.
#[must_use]
pub fn rows_to_csv(rows: &[CekUnitRow]) -> String {
    let mut out = String::new();
    out.push_str(
        "no,no_perjanjian,nama_nasabah,nopol,coll,pic,kategori,jto,\
         no_rangka,no_mesin,merk,type,warna,status,actual_penyelesaian,\
         angsuran_ke,tenor\n",
    );
    for row in rows {
        let fields = [
            &row.no,
            &row.no_perjanjian,
            &row.nama_nasabah,
            &row.nopol,
            &row.coll,
            &row.pic,
            &row.kategori,
            &row.jto,
            &row.no_rangka,
            &row.no_mesin,
            &row.merk,
            &row.r#type,
            &row.warna,
            &row.status,
            &row.actual_penyelesaian,
            &row.angsuran_ke,
            &row.tenor,
        ];
        let line = fields
            .iter()
            .map(|f| csv_escape(f))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Serialize InputUser rows to CSV text with a fixed header.
#[must_use]
pub fn input_user_rows_to_csv(rows: &[InputUserRow]) -> String {
    let mut out = String::new();
    out.push_str(
        "no,created_at,user_id,nopol,lokasi,forn,nama,kategori,nama_nasabah,no_perjanjian\n",
    );
    for row in rows {
        let fields = [
            &row.no,
            &row.created_at,
            &row.user_id,
            &row.nopol,
            &row.lokasi,
            &row.forn,
            &row.nama,
            &row.kategori,
            &row.nama_nasabah,
            &row.no_perjanjian,
        ];
        let line = fields
            .iter()
            .map(|f| csv_escape(f))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Escape a field for CSV.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_owned()
    }
}
