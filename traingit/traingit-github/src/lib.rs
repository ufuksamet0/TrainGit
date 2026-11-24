//! traingit-github - GitHub API adapter using octocrab

pub mod adapter;
pub mod conversion;

pub use adapter::GitHubAdapter;
pub use conversion::*;