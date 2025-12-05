//! Link extraction from posts

use crate::bluesky::{Facet, FacetFeature, PostRecord};

/// Extract all links from a post's facets
pub fn extract_links(record: &PostRecord) -> Vec<String> {
    let Some(facets) = &record.facets else {
        return Vec::new();
    };

    facets.iter().flat_map(extract_links_from_facet).collect()
}

fn extract_links_from_facet(facet: &Facet) -> Vec<String> {
    facet
        .features
        .iter()
        .filter_map(|feature| match feature {
            FacetFeature::Link { uri } => Some(uri.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bluesky::ByteSlice;
    use chrono::Utc;

    #[test]
    fn test_extract_links_empty() {
        let record = PostRecord {
            text: "No links here".to_string(),
            created_at: Utc::now(),
            reply: None,
            facets: None,
        };
        assert!(extract_links(&record).is_empty());
    }

    #[test]
    fn test_extract_links_with_link() {
        let record = PostRecord {
            text: "Check this out: https://example.com".to_string(),
            created_at: Utc::now(),
            reply: None,
            facets: Some(vec![Facet {
                index: ByteSlice {
                    byte_start: 16,
                    byte_end: 35,
                },
                features: vec![FacetFeature::Link {
                    uri: "https://example.com".to_string(),
                }],
            }]),
        };
        let links = extract_links(&record);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], "https://example.com");
    }
}
