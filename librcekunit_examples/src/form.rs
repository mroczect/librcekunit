use librcekunit_client::prelude::Form;

#[must_use]
pub fn form(pairs: &[(&str, &str)]) -> Form {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

#[derive(Debug, Default, Clone)]
pub struct FormBuilder {
    inner: Form,
}

impl FormBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn set(mut self, key: &str, value: &str) -> Self {
        let _ = self.inner.insert(key.to_owned(), value.to_owned());
        self
    }

    #[must_use]
    pub fn set_opt(self, key: &str, value: Option<&str>) -> Self {
        match value {
            Some(v) => self.set(key, v),
            None => self,
        }
    }

    #[must_use]
    pub fn set_nonempty(self, key: &str, value: &str) -> Self {
        if value.is_empty() {
            self
        } else {
            self.set(key, value)
        }
    }

    #[must_use]
    pub fn set_nonzero(self, key: &str, value: u64) -> Self {
        if value == 0 {
            self
        } else {
            self.set(key, &value.to_string())
        }
    }

    #[must_use]
    pub fn build(self) -> Form {
        self.inner
    }
}
