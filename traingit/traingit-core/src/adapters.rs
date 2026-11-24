//! Git service adapter traits

use async_trait::async_trait;
use crate::models::*;
use crate::errors::Result;

/// Trait for Git service adapters
#[async_trait]
pub trait GitAdapter: Send + Sync {
    /// Returns the service type this adapter handles
    fn service_type(&self) -> GitService;

    // ==================== Discovery ====================

    /// Search repositories with the given query
    async fn search_repositories(&self, query: &SearchQuery) -> Result<SearchResult>;

    /// Get a single repository by owner and name
    async fn get_repository(&self, owner: &str, name: &str) -> Result<Repository>;

    /// List repositories for an organization/user
    async fn list_repositories(&self, owner: &str, params: &ListParams) -> Result<Vec<Repository>>;

    // ==================== Collection ====================

    /// Get file content from a repository
    async fn get_file_content(
        &self,
        owner: &str,
        name: &str,
        path: &str,
        branch: &str,
    ) -> Result<String>;

    /// Get directory tree
    async fn get_directory_tree(
        &self,
        owner: &str,
        name: &str,
        branch: &str,
        path: &str,
    ) -> Result<Vec<TreeEntry>>;

    /// Get issues
    async fn get_issues(
        &self,
        owner: &str,
        name: &str,
        params: &IssueParams,
    ) -> Result<Vec<CollectedIssue>>;

    /// Get pull requests
    async fn get_pull_requests(
        &self,
        owner: &str,
        name: &str,
        params: &PRParams,
    ) -> Result<Vec<CollectedPullRequest>>;

    /// Get commits
    async fn get_commits(
        &self,
        owner: &str,
        name: &str,
        params: &CommitParams,
    ) -> Result<Vec<CollectedCommit>>;

    /// Get README content
    async fn get_readme(&self, owner: &str, name: &str, branch: &str) -> Result<Option<String>>;

    /// Get license content
    async fn get_license(&self, owner: &str, name: &str, branch: &str) -> Result<Option<String>>;

    // ==================== Rate Limiting ====================

    /// Get current rate limit status
    async fn get_rate_limit(&self) -> Result<RateLimitInfo>;

    /// Wait until rate limit resets (if needed)
    async fn wait_for_rate_limit(&self) -> Result<()>;

    // ==================== Utility ====================

    /// Check if the adapter is authenticated
    fn is_authenticated(&self) -> bool;

    /// Get the base API URL
    fn api_base_url(&self) -> &str;
}

/// Adapter factory for creating adapters
pub trait AdapterFactory: Send + Sync {
    fn create_github(&self, token: Option<String>) -> Box<dyn GitAdapter>;
    fn create_gitlab(&self, token: Option<String>, base_url: Option<String>) -> Box<dyn GitAdapter>;
    fn create_bitbucket(&self, username: Option<String>, app_password: Option<String>) -> Box<dyn GitAdapter>;
    fn create_gitea(&self, instance_url: String, token: Option<String>) -> Box<dyn GitAdapter>;
}

/// Multi-adapter for searching across multiple services
pub trait MultiAdapter: Send + Sync {
    /// Get all registered adapters
    fn adapters(&self) -> Vec<Box<dyn GitAdapter>>;

    /// Search across all adapters in parallel
    async fn search_all(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let adapters = self.adapters();
        let mut handles = Vec::new();
        
        for adapter in adapters {
            let query = query.clone();
            handles.push(tokio::spawn(async move {
                adapter.search_repositories(&query).await
            }));
        }
        
        let mut all_results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => all_results.push(result?),
                Err(e) => return Err(crate::errors::CoreError::Internal(format!("Task join error: {}", e))),
            }
        }
        Ok(all_results)
    }

    /// Find adapter for a specific service
    fn find_adapter(&self, service: &GitService) -> Option<Box<dyn GitAdapter>> {
        self.adapters().into_iter()
            .find(|a| a.service_type() == *service)
    }
}