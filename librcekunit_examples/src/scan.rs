use core::time::Duration;

use librcekunit_client::prelude::{Client, Dashboard, Error, Form, InputUser, Result};

use crate::parse::{self, CekUnitRow, InputUserRow};

/// Options controlling a scan.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Maximum number of pages to fetch.
    pub max_pages: u32,
    /// Page to start from, 1-based.
    pub start_page: u32,
    /// Delay between page requests.
    pub page_delay: Duration,
    /// Sort column. Empty means server default.
    pub sort: String,
    /// Sort direction, `asc` or `desc`.
    pub direction: String,
    /// Free-text search term. Empty means no filter.
    pub search: String,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            max_pages: 10,
            start_page: 1,
            page_delay: Duration::from_millis(150),
            sort: String::new(),
            direction: String::from("asc"),
            search: String::new(),
        }
    }
}

impl ScanOptions {
    /// Set the maximum number of pages.
    #[must_use]
    pub const fn max_pages(mut self, n: u32) -> Self {
        self.max_pages = n;
        self
    }

    /// Set the starting page.
    #[must_use]
    pub const fn start_page(mut self, n: u32) -> Self {
        self.start_page = n;
        self
    }

    /// Set the delay between page requests.
    #[must_use]
    pub const fn page_delay(mut self, d: Duration) -> Self {
        self.page_delay = d;
        self
    }

    /// Set the search term.
    #[must_use]
    pub fn search(mut self, s: impl Into<String>) -> Self {
        self.search = s.into();
        self
    }

    /// Set both the sort column and direction.
    #[must_use]
    pub fn sort(mut self, column: impl Into<String>, direction: impl Into<String>) -> Self {
        self.sort = column.into();
        self.direction = direction.into();
        self
    }
}

/// Why the scan stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScanStop {
    /// Reached the page limit set by the caller.
    MaxPagesReached,
    /// The server returned a page with no rows.
    EmptyPage,
    /// The server errored on a page.
    ServerError,
}

/// Result of scanning the dashboard table.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Rows collected across all fetched pages.
    pub rows: Vec<CekUnitRow>,
    /// Number of pages successfully fetched.
    pub pages_fetched: u32,
    /// Server-reported total, when visible.
    pub total_reported: Option<usize>,
    /// Why the scan stopped.
    pub stopped: ScanStop,
}

/// Result of scanning the input-user table.
#[derive(Debug, Clone)]
pub struct InputUserScanResult {
    /// Rows collected across all fetched pages.
    pub rows: Vec<InputUserRow>,
    /// Number of pages successfully fetched.
    pub pages_fetched: u32,
    /// Server-reported total, when visible.
    pub total_reported: Option<usize>,
    /// Why the scan stopped.
    pub stopped: ScanStop,
}

/// Progress event passed to the callback after each page.
#[derive(Debug, Clone, Copy)]
pub struct Progress {
    /// Page number just fetched.
    pub page: u32,
    /// Rows parsed on this page.
    pub rows_on_page: usize,
    /// Cumulative rows collected so far.
    pub rows_total: usize,
    /// Server-reported total, when known.
    pub total_reported: Option<usize>,
}

/// Progress event for input-user scans.
#[derive(Debug, Clone, Copy)]
pub struct InputUserProgress {
    /// Page number just fetched.
    pub page: u32,
    /// Rows parsed on this page.
    pub rows_on_page: usize,
    /// Cumulative rows collected so far.
    pub rows_total: usize,
    /// Server-reported total, when known.
    pub total_reported: Option<usize>,
}

/// Scan dashboard pages until completion or the page limit.
///
/// # Errors
///
/// Returns an error when the first page fails. Errors on later pages
/// are recorded in [`ScanResult::stopped`].
pub async fn scan_all(client: &Client, opts: &ScanOptions) -> Result<ScanResult> {
    scan_with(client, opts, |_| {}).await
}

/// Scan dashboard with a progress callback.
///
/// # Errors
///
/// See [`scan_all`].
pub async fn scan_with<F>(
    client: &Client,
    opts: &ScanOptions,
    mut on_progress: F,
) -> Result<ScanResult>
where
    F: FnMut(Progress),
{
    let mut rows: Vec<CekUnitRow> = Vec::new();
    let mut pages_fetched: u32 = 0;
    let mut total_reported: Option<usize> = None;
    let mut stopped = ScanStop::MaxPagesReached;

    let end = opts.start_page.saturating_add(opts.max_pages);
    let mut page = opts.start_page;

    while page < end {
        let html = match fetch_dashboard_page(client, opts, page).await {
            Ok(body) => body,
            Err(err) => {
                if pages_fetched == 0 {
                    return Err(err);
                }
                stopped = ScanStop::ServerError;
                break;
            }
        };

        let parsed = match parse::parse_dashboard_rows(&html) {
            Ok(p) => p,
            Err(err) => {
                if pages_fetched == 0 {
                    return Err(Error::Json(err.to_string()));
                }
                stopped = ScanStop::ServerError;
                break;
            }
        };

        if total_reported.is_none() {
            total_reported = parse::parse_total_records(&html);
        }

        if parsed.is_empty() {
            stopped = ScanStop::EmptyPage;
            break;
        }

        let rows_on_page = parsed.len();
        rows.extend(parsed);
        pages_fetched = pages_fetched.saturating_add(1);

        on_progress(Progress {
            page,
            rows_on_page,
            rows_total: rows.len(),
            total_reported,
        });

        if let Some(total) = total_reported
            && rows.len() >= total
        {
            break;
        }

        page = page.saturating_add(1);

        if !opts.page_delay.is_zero() && page < end {
            tokio::time::sleep(opts.page_delay).await;
        }
    }

    Ok(ScanResult {
        rows,
        pages_fetched,
        total_reported,
        stopped,
    })
}

/// Scan input-user pages until completion or the page limit.
///
/// # Errors
///
/// Returns an error when the first page fails. Errors on later pages
/// are recorded in [`InputUserScanResult::stopped`].
pub async fn scan_input_user_all(
    client: &Client,
    opts: &ScanOptions,
) -> Result<InputUserScanResult> {
    scan_input_user_with(client, opts, |_| {}).await
}

/// Scan input-user with a progress callback.
///
/// # Errors
///
/// See [`scan_input_user_all`].
pub async fn scan_input_user_with<F>(
    client: &Client,
    opts: &ScanOptions,
    mut on_progress: F,
) -> Result<InputUserScanResult>
where
    F: FnMut(InputUserProgress),
{
    let mut rows: Vec<InputUserRow> = Vec::new();
    let mut pages_fetched: u32 = 0;
    let mut total_reported: Option<usize> = None;
    let mut stopped = ScanStop::MaxPagesReached;

    let end = opts.start_page.saturating_add(opts.max_pages);
    let mut page = opts.start_page;

    while page < end {
        let html = match fetch_input_user_page(client, opts, page).await {
            Ok(body) => body,
            Err(err) => {
                if pages_fetched == 0 {
                    return Err(err);
                }
                stopped = ScanStop::ServerError;
                break;
            }
        };

        let parsed = match parse::parse_input_user_rows(&html) {
            Ok(p) => p,
            Err(err) => {
                if pages_fetched == 0 {
                    return Err(Error::Json(err.to_string()));
                }
                stopped = ScanStop::ServerError;
                break;
            }
        };

        if total_reported.is_none() {
            total_reported = parse::parse_total_records(&html);
        }

        if parsed.is_empty() {
            stopped = ScanStop::EmptyPage;
            break;
        }

        let rows_on_page = parsed.len();
        rows.extend(parsed);
        pages_fetched = pages_fetched.saturating_add(1);

        on_progress(InputUserProgress {
            page,
            rows_on_page,
            rows_total: rows.len(),
            total_reported,
        });

        if let Some(total) = total_reported
            && rows.len() >= total
        {
            break;
        }

        page = page.saturating_add(1);

        if !opts.page_delay.is_zero() && page < end {
            tokio::time::sleep(opts.page_delay).await;
        }
    }

    Ok(InputUserScanResult {
        rows,
        pages_fetched,
        total_reported,
        stopped,
    })
}

/// Fetch one dashboard page's HTML.
async fn fetch_dashboard_page(client: &Client, opts: &ScanOptions, page: u32) -> Result<String> {
    let mut params = Form::new();
    let _ = params.insert(String::from("page"), page.to_string());
    let _ = params.insert(String::from("direction"), opts.direction.clone());
    if !opts.sort.is_empty() {
        let _ = params.insert(String::from("sort"), opts.sort.clone());
    }
    if !opts.search.is_empty() {
        let _ = params.insert(String::from("search"), opts.search.clone());
    }

    let resp = client.dashboard_index_with_params(params).await?;
    resp.text().await.map_err(|e| Error::Network(e.to_string()))
}

/// Fetch one input-user page's HTML.
async fn fetch_input_user_page(client: &Client, opts: &ScanOptions, page: u32) -> Result<String> {
    let mut params = Form::new();
    let _ = params.insert(String::from("page"), page.to_string());
    let _ = params.insert(String::from("direction"), opts.direction.clone());
    if !opts.sort.is_empty() {
        let _ = params.insert(String::from("sort"), opts.sort.clone());
    }
    if !opts.search.is_empty() {
        let _ = params.insert(String::from("search"), opts.search.clone());
    }

    let resp = client.input_user_index_with_params(params).await?;
    resp.text().await.map_err(|e| Error::Network(e.to_string()))
}
