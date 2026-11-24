//! traingit-gitlab - GitLab API adapter

pub mod adapter;
pub mod conversion;

pub use adapter::GitLabAdapter;
pub use conversion::*;