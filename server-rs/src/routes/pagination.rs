//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / pagination
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `pagination::tests::*`

use serde::Serialize;
use std::collections::BTreeMap;

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

    /// Calculates the offset for slicing safely using saturating arithmetic.
    pub fn offset(&self) -> usize {
        let (page, per_page) = self.sanitize();
        (page.saturating_sub(1) as usize).saturating_mul(per_page as usize)
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

/// HATEOAS link object per HAL / JSON:API conventions.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
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
}

fn build_links(
    base_path: &str,
    page: u32,
    per_page: u32,
    total_pages: u32,
) -> BTreeMap<String, HateoasLink> {
    let mut links = BTreeMap::new();
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

    links
}

/// Standard paginated response envelope with HATEOAS navigation links.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
    pub _links: BTreeMap<String, HateoasLink>,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Creates a paginated response from already sliced data and a total count.
    pub fn from_pre_sliced(
        data: Vec<T>,
        total: u32,
        params: &PaginationParams,
        base_path: &str,
    ) -> Self {
        let (page, per_page) = params.sanitize();
        let total_pages = if total == 0 {
            1
        } else {
            total.div_ceil(per_page)
        };

        let links = build_links(base_path, page, per_page, total_pages);

        Self {
            data,
            page,
            per_page,
            total,
            total_pages,
            _links: links,
        }
    }

    /// Creates a paginated response from a full collection, slicing to the requested page.
    pub fn from_vec(mut items: Vec<T>, params: &PaginationParams, base_path: &str) -> Self {
        let (page, per_page) = params.sanitize();
        let total = items.len() as u32;
        let total_pages = if total == 0 {
            1
        } else {
            total.div_ceil(per_page)
        };
        let offset = params.offset();

        let data: Vec<T> = if offset >= items.len() {
            vec![]
        } else {
            items
                .drain(offset..(offset + per_page as usize).min(items.len()))
                .collect()
        };

        let links = build_links(base_path, page, per_page, total_pages);

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
    fn pagination_offset_overflow_safe() {
        let p = PaginationParams {
            page: u32::MAX,
            per_page: 100,
        };
        let offset = p.offset();
        assert!(offset > 0);
    }

    #[test]
    fn paginated_response_from_pre_sliced() {
        let items: Vec<i32> = vec![1, 2, 3];
        let params = PaginationParams {
            page: 1,
            per_page: 10,
        };
        let resp = PaginatedResponse::from_pre_sliced(items, 25, &params, "/v1/test");
        assert_eq!(resp.data.len(), 3);
        assert_eq!(resp.total, 25);
        assert_eq!(resp.total_pages, 3);
        assert!(resp._links.contains_key("self"));
        assert!(resp._links.contains_key("next"));
    }

    #[test]
    fn paginated_response_from_vec_slices_correctly() {
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
}
