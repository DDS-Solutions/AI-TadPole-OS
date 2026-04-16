//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **API Pagination (Response Envelope)**: Orchestrates the
//! standardized formatting of windowed datasets and HATEOAS link
//! generation for the Tadpole OS engine. Features **Clamped Window
//! Sanitization**: ensures that pagination parameters (page/per_page)
//! are within safe operational limits (max 100 per page). Implements
//! **RFC 8288-Compliant Navigation**: generates consistent `_links`
//! (self, first, last, next, prev) to facilitate automated
//! collection crawling by AI agents. AI agents should use the
//! `PaginatedResponse::from_vec` method to wrap all collection
//! outputs before dispatching to the frontend (NET-04).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Off-by-one errors in offset calculation,
//!   broken HATEOAS links due to incorrect `base_path` resolution,
//!   or performance degradation when slicing very large in-memory
//!   collections.
//! - **Trace Scope**: `server-rs::routes::pagination`

use serde::Serialize;

/// Standard query parameters for paginated list endpoints.
///
/// Supports `?page=1&per_page=25` — both optional with sensible defaults.
/// Max per_page is capped at 100 to prevent abuse.
#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    25
}

impl PaginationParams {
    /// Returns clamped, safe values (page >= 1, per_page in 1..=100).
    pub fn sanitize(&self) -> (u32, u32) {
        let page = self.page.max(1);
        let per_page = self.per_page.clamp(1, 100);
        (page, per_page)
    }

    /// Calculates the offset for slicing.
    pub fn offset(&self) -> usize {
        let (page, per_page) = self.sanitize();
        ((page - 1) * per_page) as usize
    }
}

/// HATEOAS link object per RFC 8288 (Web Linking).
#[derive(Debug, Clone, Serialize)]
pub struct HateoasLink {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

impl HateoasLink {
    pub fn get(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            method: Some("GET".to_string()),
        }
    }
    pub fn _post(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            method: Some("POST".to_string()),
        }
    }
    pub fn _put(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            method: Some("PUT".to_string()),
        }
    }
    pub fn _delete(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            method: Some("DELETE".to_string()),
        }
    }
}

/// Standard paginated response envelope with HATEOAS navigation links.
///
/// Example response:
/// ```json
/// {
///   "data": [...],
///   "page": 1,
///   "per_page": 25,
///   "total": 42,
///   "total_pages": 2,
///   "_links": {
///     "self": { "href": "/v1/agents?page=1&per_page=25" },
///     "next": { "href": "/v1/agents?page=2&per_page=25" },
///     "last": { "href": "/v1/agents?page=2&per_page=25" }
///   }
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
    pub _links: std::collections::HashMap<String, HateoasLink>,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Creates a paginated response from a full collection, slicing to the requested page.
    ///
    /// Automatically generates HATEOAS navigation links (self, first, last, next, prev).
    pub fn from_vec(mut items: Vec<T>, params: &PaginationParams, base_path: &str) -> Self {
        let (page, per_page) = params.sanitize();
        let total = items.len() as u32;
        let total_pages = if total == 0 {
            1
        } else {
            total.div_ceil(per_page)
        };
        let offset = params.offset();

        // Slice to requested page
        let data: Vec<T> = if offset >= items.len() {
            vec![]
        } else {
            items
                .drain(offset..(offset + per_page as usize).min(items.len()))
                .collect()
        };

        // Build HATEOAS navigation links
        let mut links = std::collections::HashMap::new();
        links.insert(
            "self".to_string(),
            HateoasLink::get(format!("{}?page={}&per_page={}", base_path, page, per_page)),
        );
        links.insert(
            "first".to_string(),
            HateoasLink::get(format!("{}?page=1&per_page={}", base_path, per_page)),
        );
        links.insert(
            "last".to_string(),
            HateoasLink::get(format!(
                "{}?page={}&per_page={}",
                base_path, total_pages, per_page
            )),
        );

        if page < total_pages {
            links.insert(
                "next".to_string(),
                HateoasLink::get(format!(
                    "{}?page={}&per_page={}",
                    base_path,
                    page + 1,
                    per_page
                )),
            );
        }
        if page > 1 {
            links.insert(
                "prev".to_string(),
                HateoasLink::get(format!(
                    "{}?page={}&per_page={}",
                    base_path,
                    page - 1,
                    per_page
                )),
            );
        }

        Self {
            data,
            page,
            per_page,
            total,
            total_pages,
            _links: links,
        }
    }
}

// Builds a HATEOAS `_links` map for a single resource.
//
// Example:
// ```rust
// let links = resource_links("/v1/agents/42", &[
//     ("self", "GET"),
//     ("update", "PUT"),
//     ("delete", "DELETE"),
//     ("send", "POST", "/v1/agents/42/send"),
// ]);
// ```
/*
pub fn resource_links(self_href: &str, extra: &[(&str, &str, &str)]) -> std::collections::HashMap<String, HateoasLink> {
    let mut links = std::collections::HashMap::new();
    links.insert("self".to_string(), HateoasLink::get(self_href));
    for (name, method, href) in extra {
        links.insert(name.to_string(), HateoasLink {
            href: href.to_string(),
            method: Some(method.to_string()),
        });
    }
    links
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_defaults() {
        let p = PaginationParams {
            page: 0,
            per_page: 0,
        };
        let (page, per_page) = p.sanitize();
        assert_eq!(page, 1);
        assert_eq!(per_page, 1);
    }

    #[test]
    fn pagination_caps_per_page() {
        let p = PaginationParams {
            page: 1,
            per_page: 500,
        };
        let (_, per_page) = p.sanitize();
        assert_eq!(per_page, 100);
    }

    #[test]
    fn paginated_response_slices_correctly() {
        let items: Vec<i32> = (1..=50).collect();
        let params = PaginationParams {
            page: 2,
            per_page: 10,
        };
        let resp = PaginatedResponse::from_vec(items, &params, "/v1/test");
        assert_eq!(resp.data, (11..=20).collect::<Vec<i32>>());
        assert_eq!(resp.total, 50);
        assert_eq!(resp.total_pages, 5);
        assert_eq!(resp.page, 2);
    }

    #[test]
    fn paginated_response_generates_hateoas_links() {
        let items: Vec<i32> = (1..=30).collect();
        let params = PaginationParams {
            page: 2,
            per_page: 10,
        };
        let resp = PaginatedResponse::from_vec(items, &params, "/v1/test");
        assert!(resp._links.contains_key("self"));
        assert!(resp._links.contains_key("next"));
        assert!(resp._links.contains_key("prev"));
        assert!(resp._links.contains_key("first"));
        assert!(resp._links.contains_key("last"));
    }

    #[test]
    fn paginated_response_no_next_on_last_page() {
        let items: Vec<i32> = (1..=10).collect();
        let params = PaginationParams {
            page: 1,
            per_page: 25,
        };
        let resp = PaginatedResponse::from_vec(items, &params, "/v1/test");
        assert!(!resp._links.contains_key("next"));
        assert!(!resp._links.contains_key("prev"));
    }

    #[test]
    fn paginated_response_empty_collection() {
        let items: Vec<i32> = vec![];
        let params = PaginationParams {
            page: 1,
            per_page: 25,
        };
        let resp = PaginatedResponse::from_vec(items, &params, "/v1/test");
        assert_eq!(resp.total, 0);
        assert_eq!(resp.total_pages, 1);
        assert!(resp.data.is_empty());
    }
}
