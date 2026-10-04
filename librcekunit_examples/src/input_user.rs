use librcekunit_client::prelude::{Client, Error, Form, InputUser, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SortColumn {
    Id,
    CreatedAt,
    UserId,
    Nopol,
    Lokasi,
    ForN,
    Nama,
}

impl SortColumn {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::CreatedAt => "created_at",
            Self::UserId => "userID",
            Self::Nopol => "nopol",
            Self::Lokasi => "lokasi",
            Self::ForN => "ForN",
            Self::Nama => "nama",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Direction {
    Asc,
    Desc,
}

impl Direction {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Query {
    pub page: u32,
    pub sort: SortColumn,
    pub direction: Direction,
    pub search: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            page: 1,
            sort: SortColumn::Id,
            direction: Direction::Asc,
            search: String::new(),
            start_date: None,
            end_date: None,
        }
    }
}

impl Query {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    #[must_use]
    pub const fn sort(mut self, sort: SortColumn) -> Self {
        self.sort = sort;
        self
    }

    #[must_use]
    pub const fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = search.into();
        self
    }

    #[must_use]
    pub fn range(mut self, start: impl Into<String>, end: impl Into<String>) -> Self {
        self.start_date = Some(start.into());
        self.end_date = Some(end.into());
        self
    }

    #[must_use]
    pub fn to_form(&self) -> Form {
        let mut form = Form::new();
        let _ = form.insert("page".into(), self.page.to_string());
        let _ = form.insert("sort".into(), self.sort.as_str().into());
        let _ = form.insert("direction".into(), self.direction.as_str().into());
        let _ = form.insert("search".into(), self.search.clone());

        if self.sort == SortColumn::CreatedAt {
            if let Some(start) = &self.start_date {
                let _ = form.insert("start_date".into(), start.clone());
            }
            if let Some(end) = &self.end_date {
                let _ = form.insert("end_date".into(), end.clone());
            }
        }

        form
    }

    #[must_use]
    pub fn to_export_form(&self, format: &str) -> Form {
        let mut form = self.to_form();
        let _ = form.insert("format".into(), format.to_owned());
        form
    }
}

pub async fn fetch_html(client: &Client, query: &Query) -> Result<String> {
    let resp = client.input_user_index_with_params(query.to_form()).await?;
    resp.text().await.map_err(|e| Error::Network(e.to_string()))
}

pub async fn export_bytes(client: &Client, query: &Query, format: &str) -> Result<Vec<u8>> {
    let resp = client
        .input_user_export(query.to_export_form(format))
        .await?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| Error::Network(e.to_string()))?;
    Ok(bytes.to_vec())
}
