use serde::Serialize;

use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct PageDataResponse {
    pub page: String,
    pub user_id: i64,
    pub status: &'static str,
}

const KNOWN_PAGES: &[&str] = &[
    "software",
    "internet",
    "processes",
    "missions",
    "hardware",
    "clan",
    "list",
    "finances",
    "university",
    "ranking",
    "log",
    "fame",
    "doom",
    "bitcoin",
    "certs",
    "mail",
    "profile",
    "settings",
    "research",
    "riddle",
    "war",
    "webserver",
    "stats",
];

pub fn page_data(
    user_id: i64,
    page: String,
) -> Result<PageDataResponse, ApiError> {
    if !KNOWN_PAGES.contains(&page.as_str()) {
        return Err(ApiError::NotFound);
    }

    Ok(PageDataResponse {
        page,
        user_id,
        status: "available",
    })
}