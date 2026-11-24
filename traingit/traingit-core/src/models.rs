//! Core data models for TrainGit

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Git hosting service type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GitService {
    GitHub,
    GitLab,
    Bitbucket,
    Gitea { instance_url: String },
}

impl GitService {
    pub fn as_str(&self) -> &str {
        match self {
            GitService::GitHub => "github",
            GitService::GitLab => "gitlab",
            GitService::Bitbucket => "bitbucket",
            GitService::Gitea { .. } => "gitea",
        }
    }

    pub fn instance_url(&self) -> Option<&str> {
        match self {
            GitService::Gitea { instance_url } => Some(instance_url),
            _ => None,
        }
    }
}

/// Repository visibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

/// Main repository model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub service: GitService,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub url: String,
    pub clone_url: String,
    pub description: Option<String>,
    pub stars: u64,
    pub forks: u64,
    pub language: Option<String>,
    pub languages: HashMap<String, u64>,
    pub topics: Vec<String>,
    pub license: Option<String>,
    pub default_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub size_kb: u64,
    pub is_fork: bool,
    pub is_archived: bool,
    pub visibility: Visibility,
}

impl Repository {
    pub fn generate_id(&self) -> String {
        format!("{}:{}/{}", self.service.as_str(), self.owner, self.name)
    }
}

/// Search query for repository discovery
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchQuery {
    pub keywords: Vec<String>,
    pub languages: Vec<String>,
    pub min_stars: Option<u64>,
    pub max_stars: Option<u64>,
    pub topics: Vec<String>,
    pub license: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub pushed_after: Option<DateTime<Utc>>,
    pub min_size_kb: Option<u64>,
    pub max_size_kb: Option<u64>,
    pub is_fork: Option<bool>,
    pub is_archived: Option<bool>,
    pub visibility: Option<Visibility>,
    pub per_page: usize,
    pub max_pages: usize,
    pub sort: SearchSort,
    pub order: SearchOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SearchSort {
    #[default]
    Stars,
    Forks,
    Updated,
    Created,
    Size,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SearchOrder {
    #[default]
    Desc,
    Asc,
}

/// Search result container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub repositories: Vec<Repository>,
    pub total_count: u64,
    pub page: usize,
    pub per_page: usize,
    pub has_next_page: bool,
}

/// Filter criteria for post-discovery filtering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterCriteria {
    pub min_stars: Option<u64>,
    pub max_stars: Option<u64>,
    pub languages: Vec<String>,
    pub excluded_languages: Vec<String>,
    pub required_topics: Vec<String>,
    pub excluded_topics: Vec<String>,
    pub licenses: Vec<String>,
    pub excluded_licenses: Vec<String>,
    pub min_size_kb: Option<u64>,
    pub max_size_kb: Option<u64>,
    pub exclude_forks: bool,
    pub exclude_archived: bool,
    pub min_activity_days: Option<u64>, // days since last push
}

/// Collected file from repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedFile {
    pub path: String,
    pub content: String,
    pub language: Option<String>,
    pub size_bytes: u64,
    pub lines: u32,
    pub hash: String, // SHA256 for dedup
    pub encoding: FileEncoding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileEncoding {
    Utf8,
    Binary,
    Other(String),
}

/// Collected issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedIssue {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub comments: Vec<IssueComment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Collected pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedPullRequest {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub merged: bool,
    pub mergeable: Option<bool>,
    pub base_branch: String,
    pub head_branch: String,
    pub commits: Vec<CommitSummary>,
    pub files_changed: Vec<PrFile>,
    pub reviews: Vec<PrReview>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrFile {
    pub path: String,
    pub status: String, // added, modified, removed, renamed
    pub additions: u32,
    pub deletions: u32,
    pub patch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReview {
    pub id: u64,
    pub author: String,
    pub state: String, // approved, changes_requested, commented
    pub body: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

/// Collected commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedCommit {
    pub sha: String,
    pub message: String,
    pub author: CommitAuthor,
    pub committer: CommitAuthor,
    pub parents: Vec<String>,
    pub stats: Option<CommitStats>,
    pub files: Vec<CommitFile>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStats {
    pub additions: u32,
    pub deletions: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFile {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    pub sha: String,
    pub message: String,
    pub author: CommitAuthor,
}

/// Tree entry for directory listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    pub mode: String,
    pub type_: TreeEntryType,
    pub size: Option<u64>,
    pub sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TreeEntryType {
    Blob,
    Tree,
    Commit,
}

/// Fully collected repository with all data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedRepository {
    pub repo: Repository,
    pub files: Vec<CollectedFile>,
    pub issues: Vec<CollectedIssue>,
    pub pull_requests: Vec<CollectedPullRequest>,
    pub commits: Vec<CollectedCommit>,
    pub readme: Option<String>,
    pub license_content: Option<String>,
    pub collected_at: DateTime<Utc>,
}

/// Training sample source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SampleSource {
    Code { file_path: String, language: String },
    Documentation { file_path: String },
    Issue { issue_number: u64 },
    PullRequest { pr_number: u64 },
    Commit { commit_sha: String },
    Readme,
    License,
}

/// Sample metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetadata {
    pub repo_id: String,
    pub repo_stars: u64,
    pub repo_language: Option<String>,
    pub tokens_estimate: u32,
    pub complexity_score: Option<f32>,
    pub tags: Vec<String>,
    pub file_extension: Option<String>,
    pub directory_depth: Option<u32>,
}

/// Final training sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub id: String,
    pub source: SampleSource,
    pub category: String,
    pub subcategory: Option<String>,
    pub content: String,
    pub metadata: SampleMetadata,
    pub quality_score: f32,
}

impl TrainingSample {
    pub fn new(
        category: String,
        source: SampleSource,
        content: String,
        metadata: SampleMetadata,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            source,
            category,
            subcategory: None,
            content,
            metadata,
            quality_score: 0.0,
        }
    }

    pub fn with_quality(mut self, score: f32) -> Self {
        self.quality_score = score;
        self
    }

    pub fn with_subcategory(mut self, subcategory: String) -> Self {
        self.subcategory = Some(subcategory);
        self
    }
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: u64,
    pub remaining: u64,
    pub reset_at: DateTime<Utc>,
    pub used: u64,
}

/// List parameters for org/user repositories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListParams {
    pub per_page: usize,
    pub page: usize,
    pub sort: Option<String>,
    pub direction: Option<String>,
    pub visibility: Option<Visibility>,
    pub affiliation: Option<String>,
}

/// Issue fetch parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueParams {
    pub state: Option<String>, // open, closed, all
    pub labels: Vec<String>,
    pub sort: Option<String>, // created, updated, comments
    pub direction: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub per_page: usize,
    pub page: usize,
}

/// Pull request fetch parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PRParams {
    pub state: Option<String>, // open, closed, merged, all
    pub sort: Option<String>, // created, updated, popularity, long-running
    pub direction: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub per_page: usize,
    pub page: usize,
}

/// Commit fetch parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitParams {
    pub sha: Option<String>,
    pub path: Option<String>,
    pub author: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub per_page: usize,
    pub page: usize,
}