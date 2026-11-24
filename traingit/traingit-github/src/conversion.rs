//! Conversion from octocrab types to traingit-core models

use octocrab::models::Repository as OctoRepository;
use octocrab::models::issues::Issue as OctoIssue;
use octocrab::models::pulls::PullRequest as OctoPullRequest;
use octocrab::models::commits::Commit as OctoCommit;
use octocrab::models::issues::Comment as OctoComment;
use octocrab::models::pulls::Review as OctoReview;
use octocrab::models::pulls::ReviewState;
use octocrab::models::repos::DiffEntry as OctoDiffEntry;
use octocrab::models::repos::RepoCommit;
use octocrab::models::{GitUser, SimpleUser, RateLimit};
use traingit_core::models::*;
use traingit_core::errors::{Result, CoreError};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

fn parse_date_str(date: &Option<String>) -> DateTime<Utc> {
    date.as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now())
}

fn git_user_to_commit_author(user: &GitUser) -> CommitAuthor {
    CommitAuthor {
        name: user.name.clone().unwrap_or_default(),
        email: user.email.clone().unwrap_or_default(),
        date: parse_date_str(&user.date),
    }
}

fn author_to_commit_author(user: &octocrab::models::Author) -> CommitAuthor {
    CommitAuthor {
        name: user.name.clone().unwrap_or_else(|| user.login.clone()),
        email: user.email.clone().unwrap_or_else(|| format!("{}@users.noreply.github.com", user.login)),
        date: Utc::now(),
    }
}

fn simple_user_to_commit_author(user: &SimpleUser) -> CommitAuthor {
    CommitAuthor {
        name: user.name.clone().unwrap_or_else(|| user.login.clone()),
        email: user.email.clone().unwrap_or_else(|| format!("{}@users.noreply.github.com", user.login)),
        date: Utc::now(),
    }
}

pub fn repo_from_octocrab(repo: OctoRepository) -> Repository {
    let service = GitService::GitHub;
    let owner = repo.owner.as_ref().map(|o| o.login.clone()).unwrap_or_default();
    let name = repo.name.clone();
    let full_name = repo.full_name.unwrap_or_else(|| format!("{}/{}", owner, name));
    let default_branch = repo.default_branch.unwrap_or_else(|| "main".to_string());
    
    // Primary language
    let primary_language = match repo.language {
        Some(serde_json::Value::String(s)) => Some(s),
        _ => None,
    };
    
    // Convert languages from bytes (if available)
    let languages: HashMap<String, u64> = HashMap::new(); // Languages need separate API call
    
    Repository {
        id: format!("github:{}/{}", owner, name),
        service,
        owner: owner.clone(),
        name: name.clone(),
        full_name,
        url: repo.html_url.map(|u| u.to_string()).unwrap_or_default(),
        clone_url: repo.clone_url.map(|u| u.to_string()).unwrap_or_else(|| format!("https://github.com/{}/{}", owner, name)),
        description: repo.description,
        stars: repo.stargazers_count.unwrap_or(0) as u64,
        forks: repo.forks_count.unwrap_or(0) as u64,
        language: primary_language,
        languages,
        topics: repo.topics.unwrap_or_default(),
        license: repo.license.as_ref().map(|l| l.spdx_id.clone()),
        default_branch,
        created_at: repo.created_at.unwrap_or_else(|| Utc::now()),
        updated_at: repo.updated_at.unwrap_or_else(|| Utc::now()),
        pushed_at: repo.pushed_at.unwrap_or_else(|| Utc::now()),
        size_kb: repo.size.unwrap_or(0) as u64,
        is_fork: repo.fork.unwrap_or(false),
        is_archived: repo.archived.unwrap_or(false),
        visibility: match repo.visibility.as_deref() {
            Some("public") => Visibility::Public,
            Some("private") => Visibility::Private,
            Some("internal") => Visibility::Internal,
            _ => Visibility::Public,
        },
    }
}

pub fn issue_from_octocrab(issue: OctoIssue) -> CollectedIssue {
    CollectedIssue {
        number: issue.number,
        title: issue.title,
        body: issue.body,
        state: format!("{:?}", issue.state),
        labels: issue.labels.into_iter().map(|l| l.name).collect(),
        assignees: issue.assignees.into_iter().map(|a| a.login).collect(),
        comments: Vec::new(),
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
    }
}

pub fn comment_from_octocrab(comment: OctoComment) -> IssueComment {
    IssueComment {
        id: comment.id.0,
        author: comment.user.login,
        body: comment.body.unwrap_or_default(),
        created_at: comment.created_at,
        updated_at: comment.updated_at.unwrap_or(comment.created_at),
    }
}

pub fn pr_from_octocrab(pr: OctoPullRequest) -> CollectedPullRequest {
    CollectedPullRequest {
        number: pr.number,
        title: pr.title.unwrap_or_default(),
        body: pr.body,
        state: pr.state.map(|s| format!("{:?}", s)).unwrap_or_else(|| "Open".to_string()),
        merged: pr.merged.unwrap_or(false),
        mergeable: pr.mergeable,
        base_branch: pr.base.ref_field.clone(),
        head_branch: pr.head.ref_field.clone(),
        commits: Vec::new(),
        files_changed: Vec::new(),
        reviews: Vec::new(),
        created_at: pr.created_at.unwrap_or_else(|| Utc::now()),
        updated_at: pr.updated_at.unwrap_or_else(|| Utc::now()),
        merged_at: pr.merged_at,
        closed_at: pr.closed_at,
    }
}

pub fn commit_from_octocrab(commit: OctoCommit) -> CollectedCommit {
    let author_date = commit.commit.author.as_ref().and_then(|a| parse_date_str(&a.date).into());
    let _committer_date = commit.commit.committer.as_ref().and_then(|c| parse_date_str(&c.date).into());
    
    let author = commit.author.as_ref().map(|u| simple_user_to_commit_author(u)).unwrap_or_else(|| {
        let name = commit.commit.author.as_ref().and_then(|a| a.name.clone()).unwrap_or_default();
        let email = commit.commit.author.as_ref().and_then(|a| a.email.clone()).unwrap_or_default();
        CommitAuthor {
            name,
            email,
            date: author_date.unwrap_or_else(|| Utc::now()),
        }
    });
    
    let committer = commit.commit.committer.as_ref().map(git_user_to_commit_author).unwrap_or_else(|| CommitAuthor {
        name: String::new(),
        email: String::new(),
        date: Utc::now(),
    });
    
    CollectedCommit {
        sha: commit.sha,
        message: commit.commit.message,
        author,
        committer,
        parents: commit.parents.into_iter().map(|p| p.sha).collect(),
        stats: commit.stats.map(|s| CommitStats {
            additions: s.additions.unwrap_or(0) as u32,
            deletions: s.deletions.unwrap_or(0) as u32,
            total: s.total.unwrap_or(0) as u32,
        }),
        files: commit.files.unwrap_or_default().into_iter().map(|f| CommitFile {
            path: f.filename,
            status: format!("{:?}", f.status),
            additions: f.additions as u32,
            deletions: f.deletions as u32,
        }).collect(),
        created_at: author_date.unwrap_or_else(|| Utc::now()),
    }
}

pub fn repo_commit_from_octocrab(commit: RepoCommit) -> CollectedCommit {
    let author_date = commit.commit.author.as_ref().and_then(|a| a.date);
    let committer_date = commit.commit.committer.as_ref().and_then(|c| c.date);
    
    let author = commit.author.as_ref().map(|u| author_to_commit_author(u)).unwrap_or_else(|| {
        let author_ca = commit.commit.author.as_ref();
        CommitAuthor {
            name: author_ca.as_ref().and_then(|a| Some(a.name.clone())).unwrap_or_default(),
            email: author_ca.as_ref().and_then(|a| a.email.clone()).unwrap_or_default(),
            date: author_date.unwrap_or_else(|| Utc::now()),
        }
    });
    
    let committer = commit.committer.as_ref().map(|u| author_to_commit_author(u)).unwrap_or_else(|| {
        let committer_ca = commit.commit.committer.as_ref();
        CommitAuthor {
            name: committer_ca.as_ref().and_then(|c| Some(c.name.clone())).unwrap_or_default(),
            email: committer_ca.as_ref().and_then(|c| c.email.clone()).unwrap_or_default(),
            date: committer_date.unwrap_or_else(|| Utc::now()),
        }
    });
    
    CollectedCommit {
        sha: commit.sha,
        message: commit.commit.message,
        author,
        committer,
        parents: commit.parents.into_iter().filter_map(|p| p.sha).collect(),
        stats: None,
        files: Vec::new(),
        created_at: author_date.unwrap_or_else(|| Utc::now()),
    }
}

pub fn review_from_octocrab(review: OctoReview) -> PrReview {
    PrReview {
        id: review.id.0,
        author: review.user.map(|u| u.login).unwrap_or_default(),
        state: match review.state {
            Some(ReviewState::Approved) => "approved",
            Some(ReviewState::ChangesRequested) => "changes_requested",
            Some(ReviewState::Commented) => "commented",
            Some(ReviewState::Dismissed) => "dismissed",
            Some(ReviewState::Pending) => "pending",
            _ => "unknown",
        }.to_string(),
        body: review.body,
        submitted_at: review.submitted_at.unwrap_or_else(|| Utc::now()),
    }
}

pub fn file_from_octocrab(file: OctoDiffEntry) -> PrFile {
    PrFile {
        path: file.filename,
        status: format!("{:?}", file.status),
        additions: file.additions as u32,
        deletions: file.deletions as u32,
        patch: file.patch,
    }
}

pub fn rate_limit_from_octocrab(rl: RateLimit) -> RateLimitInfo {
    RateLimitInfo {
        limit: rl.rate.limit as u64,
        remaining: rl.rate.remaining as u64,
        reset_at: DateTime::from_timestamp(rl.rate.reset as i64, 0).unwrap_or_else(|| Utc::now()),
        used: rl.rate.used as u64,
    }
}

pub fn convert_search_result(page: octocrab::Page<OctoRepository>) -> SearchResult {
    let repositories: Vec<Repository> = page.items.into_iter().map(repo_from_octocrab).collect();
    SearchResult {
        total_count: page.total_count.unwrap_or(0) as u64,
        repositories,
        page: 1,
        per_page: 30,
        has_next_page: page.next.is_some(),
    }
}

pub fn decode_content(content: &octocrab::models::repos::Content) -> Result<String> {
    if let Some(encoded) = &content.content {
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|e| CoreError::Internal(format!("Base64 decode error: {}", e)))?;
        Ok(String::from_utf8_lossy(&decoded).to_string())
    } else {
        Ok(String::new())
    }
}