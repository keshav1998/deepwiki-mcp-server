//! Authentication Manager for DeepWiki/Devin API Keys
//!
//! This module handles authentication for MCP HTTP requests, including
//! API key management, header construction, and session validation.

use super::{BridgeError, BridgeResult, SessionInfo};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Authentication manager for MCP HTTP clients
#[derive(Debug, Clone)]
pub struct AuthManager {
    api_key: Option<String>,
    auth_type: AuthType,
}

/// Type of authentication to use
#[derive(Debug, Clone, PartialEq)]
pub enum AuthType {
    /// No authentication required
    None,
    /// Bearer token authentication
    Bearer,
    /// Custom header authentication
    #[allow(dead_code)]
    Custom { header_name: String },
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(api_key: Option<String>) -> Self {
        let auth_type = if api_key.is_some() {
            AuthType::Bearer
        } else {
            AuthType::None
        };

        Self { api_key, auth_type }
    }

    /// Create an authentication manager with custom header
    #[allow(dead_code)]
    pub fn with_custom_header(api_key: Option<String>, header_name: String) -> Self {
        let auth_type = if api_key.is_some() {
            AuthType::Custom { header_name }
        } else {
            AuthType::None
        };

        Self { api_key, auth_type }
    }

    /// Get the API key
    #[allow(dead_code)]
    pub fn get_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// Set a new API key
    #[allow(dead_code)]
    pub fn set_api_key(&mut self, api_key: Option<String>) {
        self.api_key = api_key;

        // Update auth type based on whether we have a key
        if self.api_key.is_some() && self.auth_type == AuthType::None {
            self.auth_type = AuthType::Bearer;
        } else if self.api_key.is_none() {
            self.auth_type = AuthType::None;
        }
    }

    /// Get authentication headers for HTTP requests
    pub fn get_headers(&self, session: &SessionInfo) -> BridgeResult<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add standard headers
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        // Add User-Agent
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("zed-deepwiki-mcp/0.1.0"),
        );

        // Add session ID if available
        if !session.id.is_empty() {
            headers.insert(
                "X-Session-ID",
                HeaderValue::from_str(&session.id).map_err(|e| BridgeError::Auth {
                    message: format!("Invalid session ID: {e}"),
                })?,
            );
        }

        // Add authentication headers based on type
        match &self.auth_type {
            AuthType::None => {
                debug!("No authentication required");
            }
            AuthType::Bearer => {
                if let Some(api_key) = &self.api_key {
                    let auth_value = format!("Bearer {api_key}");
                    headers.insert(
                        "Authorization",
                        HeaderValue::from_str(&auth_value).map_err(|e| BridgeError::Auth {
                            message: format!("Invalid bearer token: {e}"),
                        })?,
                    );
                    debug!("Added Bearer authentication");
                } else {
                    warn!("Bearer auth type but no API key provided");
                }
            }
            AuthType::Custom { header_name } => {
                if let Some(api_key) = &self.api_key {
                    let header_name =
                        HeaderName::from_bytes(header_name.as_bytes()).map_err(|e| {
                            BridgeError::Auth {
                                message: format!("Invalid header name '{header_name}': {e}"),
                            }
                        })?;

                    headers.insert(
                        header_name,
                        HeaderValue::from_str(api_key).map_err(|e| BridgeError::Auth {
                            message: format!("Invalid API key value: {e}"),
                        })?,
                    );
                    debug!("Added custom header authentication");
                } else {
                    warn!("Custom auth type but no API key provided");
                }
            }
        }

        // Add endpoint-specific headers
        self.add_endpoint_specific_headers(&mut headers, session)?;

        Ok(headers)
    }

    /// Add headers specific to different endpoints
    fn add_endpoint_specific_headers(
        &self,
        headers: &mut HeaderMap,
        session: &SessionInfo,
    ) -> BridgeResult<()> {
        let endpoint = &session.endpoint;

        if endpoint.contains("mcp.deepwiki.com") {
            // DeepWiki-specific headers
            headers.insert("X-Client-Name", HeaderValue::from_static("zed-extension"));

            // DeepWiki uses SSE by default
            headers.insert(
                "Accept",
                HeaderValue::from_static("text/event-stream, application/json"),
            );
        } else if endpoint.contains("mcp.devin.ai") {
            // Devin-specific headers
            headers.insert("X-Client-Name", HeaderValue::from_static("zed-extension"));

            // Ensure we have authentication for Devin
            if self.api_key.is_none() {
                return Err(BridgeError::Auth {
                    message: "Devin endpoint requires API key authentication".to_string(),
                });
            }

            // Devin also uses SSE
            headers.insert(
                "Accept",
                HeaderValue::from_static("text/event-stream, application/json"),
            );
        }

        Ok(())
    }

    /// Validate that authentication is properly configured for the endpoint
    #[allow(dead_code)]
    pub fn validate_for_endpoint(&self, endpoint: &str) -> BridgeResult<()> {
        if endpoint.contains("mcp.devin.ai") && self.api_key.is_none() {
            return Err(BridgeError::Auth {
                message: "Devin AI endpoint requires an API key for authentication".to_string(),
            });
        }

        if endpoint.contains("mcp.deepwiki.com") {
            // DeepWiki is free and doesn't require authentication
            debug!("Using free DeepWiki endpoint");
        }

        Ok(())
    }

    /// Check if the current authentication is valid
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        match &self.auth_type {
            AuthType::None => true,
            AuthType::Bearer | AuthType::Custom { .. } => self.api_key.is_some(),
        }
    }

    /// Get authentication summary for debugging
    #[allow(dead_code)]
    pub fn get_auth_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();

        summary.insert("type".to_string(), format!("{:?}", self.auth_type));
        summary.insert(
            "has_api_key".to_string(),
            self.api_key.is_some().to_string(),
        );
        summary.insert("valid".to_string(), self.is_valid().to_string());

        if let Some(key) = &self.api_key {
            // Only show first 8 characters for security
            let masked_key = if key.len() > 8 {
                format!("{}...", &key[..8])
            } else {
                "***".to_string()
            };
            summary.insert("api_key_preview".to_string(), masked_key);
        }

        summary
    }

    /// Create authentication for different endpoint types
    #[allow(dead_code)]
    pub fn for_endpoint(endpoint: &str, api_key: Option<String>) -> BridgeResult<Self> {
        let auth_manager = if endpoint.contains("mcp.devin.ai") {
            // Devin requires API key
            if api_key.is_none() {
                return Err(BridgeError::Auth {
                    message: "Devin endpoint requires API key".to_string(),
                });
            }
            Self::new(api_key)
        } else if endpoint.contains("mcp.deepwiki.com") {
            // DeepWiki is free
            Self::new(api_key) // API key is optional for DeepWiki
        } else {
            // Custom endpoint - assume bearer auth if key provided
            Self::new(api_key)
        };

        auth_manager.validate_for_endpoint(endpoint)?;
        Ok(auth_manager)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager_creation() {
        let auth = AuthManager::new(None);
        assert_eq!(auth.auth_type, AuthType::None);
        assert!(auth.is_valid()); // AuthType::None is valid for free endpoints
    }

    #[test]
    fn test_auth_manager_with_api_key() {
        let auth = AuthManager::new(Some("test-key".to_string()));
        assert_eq!(auth.auth_type, AuthType::Bearer);
        assert!(auth.is_valid());
    }

    #[test]
    fn test_custom_header_auth() {
        let auth =
            AuthManager::with_custom_header(Some("test-key".to_string()), "X-API-Key".to_string());
        assert_eq!(
            auth.auth_type,
            AuthType::Custom {
                header_name: "X-API-Key".to_string()
            }
        );
        assert!(auth.is_valid());
    }

    #[test]
    fn test_devin_endpoint_validation() {
        let result = AuthManager::for_endpoint("https://mcp.devin.ai", None);
        assert!(result.is_err());

        let result =
            AuthManager::for_endpoint("https://mcp.devin.ai", Some("valid-key".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_deepwiki_endpoint_validation() {
        let result = AuthManager::for_endpoint("https://mcp.deepwiki.com", None);
        assert!(result.is_ok());

        let result =
            AuthManager::for_endpoint("https://mcp.deepwiki.com", Some("optional-key".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_headers_generation() {
        let auth = AuthManager::new(Some("test-api-key".to_string()));
        let session = SessionInfo::new("https://mcp.deepwiki.com".to_string(), None);

        let headers = auth.get_headers(&session).unwrap();

        assert!(headers.contains_key("Content-Type"));
        assert!(headers.contains_key("Accept"));
        assert!(headers.contains_key("Authorization"));
        assert!(headers.contains_key("User-Agent"));
    }

    #[test]
    fn test_auth_summary() {
        let auth = AuthManager::new(Some("test-api-key-12345".to_string()));
        let summary = auth.get_auth_summary();

        assert_eq!(summary.get("type").unwrap(), "Bearer");
        assert_eq!(summary.get("has_api_key").unwrap(), "true");
        assert_eq!(summary.get("valid").unwrap(), "true");
        assert!(summary.contains_key("api_key_preview"));
    }
}
