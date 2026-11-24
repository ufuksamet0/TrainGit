//! GitHub API adapter implementation

use async_trait::async_trait;
use octocrab::Octocrab;
use octocrab::params::State;
use traingit_core::models::*;
use traingit_core::errors::{Result, CoreError};
use traingit_core::adapters::GitAdapter;
use std::sync::Arc;
use tracing::{debug, warn};
use crate::conversion::*;

pub struct GitHubAdapter {
    client: Arc<Octocrab>,
    token: Option<String>,
}

impl GitHubAdapter {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut builder = Octocrab::builder();
        
        if let Some(token) = &token {
            builder = builder.personal_token(token.clone());
        }
        
        let client = builder.build()
            .map_err(|e| CoreError::Internal(format!("Octocrab build error: {}", e)))?;
        
        Ok(Self {
            client: Arc::new(client),
            token,
        })
    }
    
    pub fn client(&self) -> &Octocrab {
        &self.client
    }
    
    fn build_search_query(&self, query: &SearchQuery) -> String {
        let mut parts = Vec::new();
        
        // Keywords
        for kw in &query.keywords {
            parts.push(format!("{} in:name,description,readme", kw));
        }
        
        // Languages
        for lang in &query.languages {
            parts.push(format!("language:{}", lang));
        }
        
        // Stars
        if let Some(min) = query.min_stars {
            parts.push(format!("stars:>={}", min));
        }
        if let Some(max) = query.max_stars {
            parts.push(format!("stars:<={}", max));
        }
        
        // Topics
        for topic in &query.topics {
            parts.push(format!("topic:{}", topic));
        }
        
        // License
        if let Some(license) = &query.license {
            parts.push(format!("license:{}", license));
        }
        
        // Date ranges
        if let Some(from) = query.date_from {
            parts.push(format!("created:>={}", from.format("%Y-%m-%d")));
        }
        if let Some(to) = query.date_to {
            parts.push(format!("created:<={}", to.format("%Y-%m-%d")));
        }
        if let Some(pushed) = query.pushed_after {
            parts.push(format!("pushed:>={}", pushed.format("%Y-%m-%d")));
        }
        
        // Size
        if let Some(min) = query.min_size_kb {
            parts.push(format!("size:>={}", min));
        }
        if let Some(max) = query.max_size_kb {
            parts.push(format!("size:<={}", max));
        }
        
        // Fork/archived
        if query.is_fork == Some(false) {
            parts.push("fork:false".to_string());
        }
        if query.is_archived == Some(false) {
            parts.push("archived:false".to_string());
        }
        
        // Visibility
        if let Some(vis) = &query.visibility {
            match vis {
                Visibility::Public => parts.push("is:public".to_string()),
                Visibility::Private => parts.push("is:private".to_string()),
                _ => {}
            }
        }
        
        parts.join(" ")
    }
    
    fn handle_error<T>(result: octocrab::Result<T>) -> Result<T> {
        result.map_err(|e| CoreError::Internal(e.to_string()))
    }
}

#[async_trait]
impl GitAdapter for GitHubAdapter {
    fn service_type(&self) -> GitService {
        GitService::GitHub
    }
    
    async fn search_repositories(&self, query: &SearchQuery) -> Result<SearchResult> {
        let search_query = self.build_search_query(query);
        debug!("GitHub search query: {}", search_query);
        
        let mut search = self.client.search()
            .repositories(&search_query)
            .per_page(query.per_page.min(100) as u8);
        
        // Apply sort
        match query.sort {
            SearchSort::Stars => search = search.sort("stars"),
            SearchSort::Forks => search = search.sort("forks"),
            SearchSort::Updated => search = search.sort("updated"),
            SearchSort::Created => search = search.sort("created"),
            SearchSort::Size => {} // Not directly supported
        }
        
        match query.order {
            SearchOrder::Desc => search = search.order("desc"),
            SearchOrder::Asc => search = search.order("asc"),
        }
        
        let page = Self::handle_error(search.send().await)?;
        Ok(convert_search_result(page))
    }
    
    async fn get_repository(&self, owner: &str, name: &str) -> Result<Repository> {
        let repo = Self::handle_error(self.client.repos(owner, name).get().await)?;
        Ok(repo_from_octocrab(repo))
    }
    
    async fn list_repositories(&self, _owner: &str, _params: &ListParams) -> Result<Vec<Repository>> {
        // Note: octocrab doesn't have a direct list_repos for org/user
        // We'd need to use the orgs or users API, but for now return empty
        Ok(Vec::new())
    }
    
    async fn get_file_content(
        &self,
        owner: &str,
        name: &str,
        path: &str,
        branch: &str,
    ) -> Result<String> {
        let content = Self::handle_error(
            self.client
                .repos(owner, name)
                .get_content()
                .path(path)
                .r#ref(branch)
                .send()
                .await
        )?;
        
        // ContentItems can be a file or directory
        match content.items.first() {
            Some(item) => decode_content(item),
            None => Err(CoreError::NotFound(format!("File not found: {}", path))),
        }
    }
    
    async fn get_directory_tree(
        &self,
        owner: &str,
        name: &str,
        branch: &str,
        path: &str,
    ) -> Result<Vec<TreeEntry>> {
        // Get the tree SHA for the branch first
        let repo = Self::handle_error(self.client.repos(owner, name).get().await)?;
        let default_branch = repo.default_branch.unwrap_or_else(|| "main".to_string());
        let ref_name = if branch == default_branch { "HEAD".to_string() } else { format!("refs/heads/{}", branch) };
        
        // Get the commit to get the tree SHA
        let commit = Self::handle_error(
            self.client
                .repos(owner, name)
                .get_commit(&ref_name)
                .await
        )?;
        
        let tree_sha = commit.tree.sha;
        
        let tree = Self::handle_error(
            self.client
                .git(owner, name)
                .get_tree(&tree_sha)
                .recursive(true)
                .send()
                .await
        )?;
        
        let entries = tree.tree
            .into_iter()
            .filter(|entry| entry.path.starts_with(path))
            .map(|entry| TreeEntry {
                path: entry.path,
                mode: entry.mode,
                type_: match entry.r#type.as_str() {
                    "blob" => TreeEntryType::Blob,
                    "tree" => TreeEntryType::Tree,
                    "commit" => TreeEntryType::Commit,
                    _ => TreeEntryType::Blob,
                },
                size: entry.size.map(|s| s as u64),
                sha: Some(entry.sha),
            })
            .collect();
        
        Ok(entries)
    }
    
    async fn get_issues(
        &self,
        owner: &str,
        name: &str,
        params: &IssueParams,
    ) -> Result<Vec<CollectedIssue>> {
        let state = match params.state.as_deref() {
            Some("open") => State::Open,
            Some("closed") => State::Closed,
            _ => State::All,
        };
        
        let page = Self::handle_error(
            self.client
                .issues(owner, name)
                .list()
                .state(state)
                .per_page(params.per_page.min(100) as u8)
                .page(params.page as u32)
                .send()
                .await
        )?;
        
        let mut issues = Vec::new();
        for issue in page.items {
            // Skip pull requests (they appear as issues too)
            if issue.pull_request.is_some() {
                continue;
            }
            
            let mut collected = issue_from_octocrab(issue);
            
            // Fetch comments
            if let Ok(comments_page) = Self::handle_error(
                self.client
                    .issues(owner, name)
                    .list_comments(collected.number)
                    .per_page(100)
                    .send()
                    .await
            ) {
                collected.comments = comments_page.items.into_iter().map(comment_from_octocrab).collect();
            }
            
            issues.push(collected);
        }
        
        Ok(issues)
    }
    
    async fn get_pull_requests(
        &self,
        owner: &str,
        name: &str,
        params: &PRParams,
    ) -> Result<Vec<CollectedPullRequest>> {
        let state = match params.state.as_deref() {
            Some("open") => State::Open,
            Some("closed") => State::Closed,
            Some("merged") => State::All,
            _ => State::All,
        };
        
        let page = Self::handle_error(
            self.client
                .pulls(owner, name)
                .list()
                .state(state)
                .per_page(params.per_page.min(100) as u8)
                .page(params.page as u32)
                .send()
                .await
        )?;
        
        let mut prs = Vec::new();
        for pr in page.items {
            let mut collected = pr_from_octocrab(pr);
            
            // Filter merged if requested
            if params.state.as_deref() == Some("merged") && !collected.merged {
                continue;
            }
            
            // Fetch files
            if let Ok(files_page) = Self::handle_error(
                self.client
                    .pulls(owner, name)
                    .list_files(collected.number)
                    .await
            ) {
                collected.files_changed = files_page.items.into_iter().map(file_from_octocrab).collect();
            }
            
            // Fetch reviews
            if let Ok(reviews_page) = Self::handle_error(
                self.client
                    .pulls(owner, name)
                    .list_reviews(collected.number)
                    .per_page(100)
                    .send()
                    .await
            ) {
                collected.reviews = reviews_page.items.into_iter().map(review_from_octocrab).collect();
            }
            
            prs.push(collected);
        }
        
        Ok(prs)
    }
    
    async fn get_commits(
        &self,
        owner: &str,
        name: &str,
        params: &CommitParams,
    ) -> Result<Vec<CollectedCommit>> {
        let repo_handler = self.client.repos(owner, name);
        let mut list = repo_handler
            .list_commits()
            .per_page(params.per_page.min(100) as u8)
            .page(params.page as u32);
        
        if let Some(sha) = &params.sha {
            list = list.sha(sha);
        }
        if let Some(path) = &params.path {
            list = list.path(path);
        }
        if let Some(author) = &params.author {
            list = list.author(author);
        }
        if let Some(since) = params.since {
            list = list.since(since);
        }
        if let Some(until) = params.until {
            list = list.until(until);
        }
        
        let page = Self::handle_error(list.send().await)?;
        
        Ok(page.items.into_iter().map(repo_commit_from_octocrab).collect())
    }
    
    async fn get_readme(&self, owner: &str, name: &str, branch: &str) -> Result<Option<String>> {
        match Self::handle_error(
            self.client
                .repos(owner, name)
                .get_readme()
                .r#ref(branch)
                .send()
                .await
        ) {
            Ok(readme) => {
                let content = decode_content(&readme)?;
                Ok(Some(content))
            }
            Err(_) => Ok(None),
        }
    }
    
    async fn get_license(&self, owner: &str, name: &str, _branch: &str) -> Result<Option<String>> {
        match Self::handle_error(
            self.client
                .repos(owner, name)
                .license()
                .await
        ) {
            Ok(license_content) => {
                let content = decode_content(&license_content)?;
                Ok(Some(content))
            }
            Err(_) => Ok(None),
        }
    }
    
    async fn get_rate_limit(&self) -> Result<RateLimitInfo> {
        let rate_limit = Self::handle_error(self.client.ratelimit().get().await)?;
        Ok(rate_limit_from_octocrab(rate_limit))
    }
    
    async fn wait_for_rate_limit(&self) -> Result<()> {
        let rate_limit = self.get_rate_limit().await?;
        
        if rate_limit.remaining < 10 {
            let wait_seconds = (rate_limit.reset_at - chrono::Utc::now()).num_seconds().max(1) as u64;
            warn!("GitHub rate limit low ({} remaining), waiting {} seconds", rate_limit.remaining, wait_seconds);
            tokio::time::sleep(tokio::time::Duration::from_secs(wait_seconds)).await;
        }
        
        Ok(())
    }
    
    fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
    
    fn api_base_url(&self) -> &str {
        "https://api.github.com"
    }
}