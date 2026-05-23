use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 { 1 }
fn default_per_page() -> u64 { 20 }

impl PaginationQuery {
    pub fn offset(&self) -> i64 {
        (self.page.saturating_sub(1) * self.per_page) as i64
    }

    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, query: &PaginationQuery) -> Self {
        let total_pages = ((total as f64) / (query.per_page as f64)).ceil() as u64;
        Self { items, total, page: query.page, per_page: query.per_page, total_pages }
    }
}
