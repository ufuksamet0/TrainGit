//! GitLab API adapter implementation (simplified)

use async_trait::async_trait;
use gitlab::{GitlabBuilder, api::{self, projects, AsyncQuery, Pagination}};
use gitlab::api::common::{VisibilityLevel, SortOrder};
use gitlab::api::projects::repository::{files, Tree};
use gitlab::api::projects::repository::commits as repo_commits;
use traingit_core::models::*;
use traingit_core::errors::{Result, CoreError};
use traingit_core::adapters::GitAdapter;
use std::sync::Arc;
use crate::conversion::*;

pub struct GitLabAdapter {
    client: Arc<gitlab::AsyncGitlab>,
    base_url: String,
    token: Option<String>,
}

impl GitLabAdapter {
    pub async fn new(token: Option<String>, base_url: Option<String>) -> Result<Self> {
        let host = base_url.unwrap_or_else(|| "https://gitlab.com".to_string());
        let host = host.trim_end_matches('/').to_string();
        
        let mut builder = if let Some(token) = &token {
            GitlabBuilder::new(&host, token)
        } else {
            GitlabBuilder::new_unauthenticated(&host)
        };
        
        let client = builder.build_async()
            .await
            .map_err(|e| CoreError::Internal(format!("GitLab client error: {}", e)))?;
        
        Ok(Self {
            client: Arc::new(client),
            base_url: host,
            token,
        })
    }
    
    pub fn client(&self) -> &gitlab::AsyncGitlab {
        &self.client
    }
    
    fn build_projects_query(&self, query: &SearchQuery) -> projects::ProjectsBuilder {
        let mut builder = projects::Projects::builder();
        
        // Search query
        if !query.keywords.is_empty() {
            builder.search(query.keywords.join(" "));
        }
        
        // Topics
        for topic in &query.topics {
            builder.topic(topic.clone());
        }
        
        // Archived
        if query.is_archived == Some(false) {
            builder.archived(false);
        }
        
        // Visibility
        if let Some(vis) = &query.visibility {
            builder.visibility(match vis {
                Visibility::Public => VisibilityLevel::Public,
                Visibility::Private => VisibilityLevel::Private,
                Visibility::Internal => VisibilityLevel::Internal,
            });
        }
        
        // Date ranges
        if let Some(from) = query.date_from {
            builder.updated_after(from);
        }
        if let Some(to) = query.date_to {
            builder.updated_before(to);
        }
        
        // Sort
        match query.sort {
            SearchSort::Stars => { builder.order_by(projects::ProjectOrderBy::StarCount); }
            SearchSort::Forks => {} // Not directly supported
            SearchSort::Updated => { builder.order_by(projects::ProjectOrderBy::UpdatedAt); }
            SearchSort::Created => { builder.order_by(projects::ProjectOrderBy::CreatedAt); }
            SearchSort::Size => { builder.order_by(projects::ProjectOrderBy::RepositorySize); }
        }
        
        match query.order {
            SearchOrder::Desc => { builder.sort(SortOrder::Descending); }
            SearchOrder::Asc => { builder.sort(SortOrder::Ascending); }
        }
        
        builder
    }
}

#[async_trait]
impl GitAdapter for GitLabAdapter {
    fn service_type(&self) -> GitService {
        GitService::GitLab
    }
    
    async fn search_repositories(&self, query: &SearchQuery) -> Result<SearchResult> {
        let builder = self.build_projects_query(query);
        let endpoint = builder.build()
            .map_err(|e| CoreError::Internal(format!("GitLab query build error: {}", e)))?;
        
        // Execute query with pagination
        let paged = api::paged(endpoint, Pagination::Limit(query.per_page.min(100)));
        let projects: Vec<GitLabProject> = paged.query_async(self.client.as_ref()).await
            .map_err(|e| CoreError::Internal(format!("GitLab search error: {}", e)))?;
        
        let repositories: Vec<Repository> = projects.into_iter().map(project_from_gitlab).collect();
        
        Ok(SearchResult {
            total_count: repositories.len() as u64,
            repositories,
            page: 1,
            per_page: query.per_page,
            has_next_page: false,
        })
    }
    
    async fn get_repository(&self, owner: &str, name: &str) -> Result<Repository> {
        let project_path = format!("{}/{}", owner, name);
        let endpoint = projects::Project::builder()
            .project(project_path)
            .statistics(true)
            .license(true)
            .build()
            .map_err(|e| CoreError::Internal(format!("GitLab project endpoint error: {}", e)))?;
        
        let project: GitLabProject = endpoint.query_async(self.client.as_ref()).await
            .map_err(|e| CoreError::Internal(format!("GitLab get project error: {}", e)))?;
        
        Ok(project_from_gitlab(project))
    }
    
    async fn list_repositories(&self, _owner: &str, params: &ListParams) -> Result<Vec<Repository>> {
        let endpoint = projects::Projects::builder()
            .membership(true)
            .owned(true)
            .build()
            .map_err(|e| CoreError::Internal(format!("GitLab list projects error: {}", e)))?;
        
        let paged = api::paged(endpoint, Pagination::Limit(params.per_page.min(100)));
        let projects: Vec<GitLabProject> = paged.query_async(self.client.as_ref()).await
            .map_err(|e| CoreError::Internal(format!("GitLab list projects query error: {}", e)))?;
        
        Ok(projects.into_iter().map(project_from_gitlab).collect())
    }
    
    async fn get_file_content(
        &self,
        owner: &str,
        name: &str,
        path: &str,
        branch: &str,
    ) -> Result<String> {
        let project_path = format!("{}/{}", owner, name);
        let endpoint = files::File::builder()
            .project(project_path)
            .file_path(path)
            .ref_(branch)
            .build()
            .map_err(|e| CoreError::Internal(format!("GitLab file endpoint error: {}", e)))?;
        
        let file: GitLabFile = endpoint.query_async(self.client.as_ref()).await
            .map_err(|e| CoreError::Internal(format!("GitLab get file error: {}", e)))?;
        
        if let Some(content) = file.content {
            if file.encoding == "base64" {
                let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &content)
                    .map_err(|e| CoreError::Internal(format!("Base64 decode error: {}", e)))?;
                Ok(String::from_utf8_lossy(&decoded).to_string())
            } else {
                Ok(content)
            }
        } else {
            Ok(String::new())
        }
    }
    
    async fn get_directory_tree(
        &self,
        owner: &str,
        name: &str,
        branch: &str,
        path: &str,
    ) -> Result<Vec<TreeEntry>> {
        let project_path = format!("{}/{}", owner, name);
        let endpoint = Tree::builder()
            .project(project_path)
            .ref_(branch)
            .path(path)
            .recursive(true)
            .build()
            .map_err(|e| CoreError::Internal(format!("GitLab tree endpoint error: {}", e)))?;
        
        let tree: Vec<GitLabTreeNode> = endpoint.query_async(self.client.as_ref()).await
            .map_err(|e| CoreError::Internal(format!("GitLab get tree error: {}", e)))?;
        
        Ok(tree.into_iter().map(tree_node_from_gitlab).collect())
    }
    
    async fn get_issues(
        &self,
        _owner: &str,
        _name: &str,
        _params: &IssueParams,
    ) -> Result<Vec<CollectedIssue>> {
        // Simplified - GitLab issues API is complex (ProjectIssues)
        // Return empty for now
        Ok(Vec::new())
    }
    
    async fn get_pull_requests(
        &self,
        _owner: &str,
        _name: &str,
        _params: &PRParams,
    ) -> Result<Vec<CollectedPullRequest>> {
        // Simplified - GitLab MR API is complex (ProjectMergeRequests)
        // Return empty for now
        Ok(Vec::new())
    }
    
    async fn get_commits(
        &self,
        owner: &str,
        name: &str,
        params: &CommitParams,
    ) -> Result<Vec<CollectedCommit>> {
        let project_path = format!("{}/{}", owner, name);
        
        let endpoint = repo_commits::Commits::builder()
            .project(project_path)
            .ref_name(params.sha.clone())
            .since(params.since)
            .until(params.until)
            .path(params.path.clone())
            .author(params.author.clone())
            .build()
            .map_err(|e| CoreError::Internal(format!("GitLab commits endpoint error: {}", e)))?;
        
        let paged = api::paged(endpoint, Pagination::Limit(params.per_page.min(100)));
        let commits: Vec<GitLabCommit> = paged.query_async(self.client.as_ref()).await
            .map_err(|e| CoreError::Internal(format!("GitLab get commits error: {}", e)))?;
        
        Ok(commits.into_iter().map(commit_from_gitlab).collect())
    }
    
    async fn get_readme(&self, owner: &str, name: &str, branch: &str) -> Result<Option<String>> {
        let content = self.get_file_content(owner, name, "README.md", branch).await?;
        if content.is_empty() {
            let content2 = self.get_file_content(owner, name, "README.rst", branch).await?;
            if content2.is_empty() {
                Ok(None)
            } else {
                Ok(Some(content2))
            }
        } else {
            Ok(Some(content))
        }
    }
    
    async fn get_license(&self, owner: &str, name: &str, _branch: &str) -> Result<Option<String>> {
        let project_path = format!("{}/{}", owner, name);
        let endpoint = projects::Project::builder()
            .project(project_path)
            .license(true)
            .build()
            .map_err(|e| CoreError::Internal(format!("GitLab license endpoint error: {}", e)))?;
        
        let project: GitLabProject = endpoint.query_async(self.client.as_ref()).await
            .map_err(|e| CoreError::Internal(format!("GitLab get license error: {}", e)))?;
        
        Ok(project.license.and_then(|l| l.html_url))
    }
    
    async fn get_rate_limit(&self) -> Result<RateLimitInfo> {
        // GitLab doesn't have a dedicated rate limit endpoint in this crate
        // Return a default
        Ok(RateLimitInfo {
            limit: 2000,
            remaining: 2000,
            reset_at: chrono::Utc::now() + chrono::Duration::hours(1),
            used: 0,
        })
    }
    
    async fn wait_for_rate_limit(&self) -> Result<()> {
        // Simplified - GitLab rate limits are per-minute
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        Ok(())
    }
    
    fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
    
    fn api_base_url(&self) -> &str {
        &self.base_url
    }
}