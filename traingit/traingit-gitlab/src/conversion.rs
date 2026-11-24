//! Conversion from GitLab API types to traingit-core models

use serde::{Deserialize, Serialize};
use traingit_core::models::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub path_with_namespace: String,
    pub description: Option<String>,
    pub visibility: String,
    pub star_count: u64,
    pub forks_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub archived: bool,
    pub web_url: String,
    pub http_url_to_repo: String,
    pub ssh_url_to_repo: String,
    pub default_branch: String,
    pub tag_list: Vec<String>,
    pub topics: Vec<String>,
    pub license: Option<GitLabLicense>,
    pub owner: Option<GitLabOwner>,
    pub namespace: Option<GitLabNamespace>,
    pub statistics: Option<GitLabStatistics>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabLicense {
    pub key: String,
    pub name: String,
    pub nickname: Option<String>,
    pub html_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabOwner {
    pub id: u64,
    pub username: String,
    pub name: String,
    pub state: String,
    pub avatar_url: Option<String>,
    pub web_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabNamespace {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub full_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabStatistics {
    pub commit_count: Option<u64>,
    pub storage_size: Option<u64>,
    pub repository_size: Option<u64>,
    pub lfs_objects_size: Option<u64>,
    pub job_artifacts_size: Option<u64>,
    pub packages_size: Option<u64>,
    pub snippets_size: Option<u64>,
    pub wiki_size: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabIssue {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub labels: Vec<String>,
    pub author: GitLabUser,
    pub assignees: Vec<GitLabUser>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabUser {
    pub id: u64,
    pub username: String,
    pub name: String,
    pub state: String,
    pub avatar_url: Option<String>,
    pub web_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabNote {
    pub id: u64,
    pub body: String,
    pub author: GitLabUser,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub system: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabMergeRequest {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub merge_status: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author: GitLabUser,
    pub assignees: Vec<GitLabUser>,
    pub reviewers: Vec<GitLabUser>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_by: Option<GitLabUser>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabCommit {
    pub id: String,
    pub short_id: String,
    pub title: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: DateTime<Utc>,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: DateTime<Utc>,
    pub message: String,
    pub parent_ids: Vec<String>,
    pub stats: Option<GitLabCommitStats>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabCommitStats {
    pub additions: u64,
    pub deletions: u64,
    pub total: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabTreeNode {
    pub id: String,
    pub name: String,
    pub type_: String,
    pub path: String,
    pub mode: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabFile {
    pub file_name: String,
    pub file_path: String,
    pub size: u64,
    pub encoding: String,
    pub content: Option<String>,
    pub content_sha256: String,
    pub ref_: String,
    pub blob_id: String,
    pub commit_id: String,
    pub last_commit_id: String,
}

pub fn project_from_gitlab(project: GitLabProject) -> Repository {
    let owner = project.owner.as_ref().map(|o| o.username.clone()).unwrap_or_else(|| 
        project.namespace.as_ref().map(|n| n.path.clone()).unwrap_or_default()
    );
    let name = project.name.clone();
    
    let languages: HashMap<String, u64> = HashMap::new();
    
    Repository {
        id: format!("gitlab:{}/{}", owner, name),
        service: GitService::GitLab,
        owner: owner.clone(),
        name: name.clone(),
        full_name: project.path_with_namespace,
        url: project.web_url,
        clone_url: project.http_url_to_repo,
        description: project.description,
        stars: project.star_count,
        forks: project.forks_count,
        language: None,
        languages,
        topics: project.topics,
        license: project.license.as_ref().map(|l| l.key.clone()),
        default_branch: project.default_branch,
        created_at: project.created_at,
        updated_at: project.updated_at,
        pushed_at: project.last_activity_at.unwrap_or(project.updated_at),
        size_kb: project.statistics.as_ref().and_then(|s| s.repository_size).unwrap_or(0) / 1024,
        is_fork: false,
        is_archived: project.archived,
        visibility: match project.visibility.as_str() {
            "public" => Visibility::Public,
            "private" => Visibility::Private,
            "internal" => Visibility::Internal,
            _ => Visibility::Public,
        },
    }
}

pub fn issue_from_gitlab(issue: GitLabIssue) -> CollectedIssue {
    CollectedIssue {
        number: issue.iid,
        title: issue.title,
        body: issue.description,
        state: issue.state,
        labels: issue.labels,
        assignees: issue.assignees.into_iter().map(|a| a.username).collect(),
        comments: Vec::new(),
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
    }
}

pub fn note_from_gitlab(note: GitLabNote) -> IssueComment {
    IssueComment {
        id: note.id,
        author: note.author.username,
        body: note.body,
        created_at: note.created_at,
        updated_at: note.updated_at,
    }
}

pub fn merge_request_from_gitlab(mr: GitLabMergeRequest) -> CollectedPullRequest {
    let state = mr.state.clone();
    CollectedPullRequest {
        number: mr.iid,
        title: mr.title,
        body: mr.description,
        state: state.clone(),
        merged: state == "merged",
        mergeable: Some(mr.merge_status == "can_be_merged"),
        base_branch: mr.target_branch,
        head_branch: mr.source_branch,
        commits: Vec::new(),
        files_changed: Vec::new(),
        reviews: Vec::new(),
        created_at: mr.created_at,
        updated_at: mr.updated_at,
        merged_at: mr.merged_at,
        closed_at: mr.closed_at,
    }
}

pub fn commit_from_gitlab(commit: GitLabCommit) -> CollectedCommit {
    let author = CommitAuthor {
        name: commit.author_name,
        email: commit.author_email,
        date: commit.author_date,
    };
    
    let committer = CommitAuthor {
        name: commit.committer_name,
        email: commit.committer_email,
        date: commit.committer_date,
    };
    
    CollectedCommit {
        sha: commit.id,
        message: commit.message,
        author,
        committer,
        parents: commit.parent_ids,
        stats: commit.stats.map(|s| CommitStats {
            additions: s.additions as u32,
            deletions: s.deletions as u32,
            total: s.total as u32,
        }),
        files: Vec::new(),
        created_at: commit.author_date,
    }
}

pub fn tree_node_from_gitlab(node: GitLabTreeNode) -> TreeEntry {
    TreeEntry {
        path: node.path,
        mode: node.mode,
        type_: match node.type_.as_str() {
            "blob" => TreeEntryType::Blob,
            "tree" => TreeEntryType::Tree,
            "commit" => TreeEntryType::Commit,
            _ => TreeEntryType::Blob,
        },
        size: None,
        sha: Some(node.id),
    }
}