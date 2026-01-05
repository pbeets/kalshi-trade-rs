//! Internal query string builder.

/// Helper to build URL query strings with proper encoding.
pub(crate) struct QueryBuilder {
    pairs: Vec<(String, String)>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn push(&mut self, key: &str, value: impl ToString) {
        self.pairs.push((key.to_string(), value.to_string()));
    }

    pub fn push_opt<T: ToString>(&mut self, key: &str, value: Option<T>) {
        if let Some(v) = value {
            self.push(key, v);
        }
    }

    pub fn build(self) -> String {
        if self.pairs.is_empty() {
            return String::new();
        }
        let encoded: String = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(self.pairs)
            .finish();
        format!("?{}", encoded)
    }
}
