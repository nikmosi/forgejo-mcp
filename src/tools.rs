use base64::prelude::*;
use serde_json::{json, Value};
use tracing::debug;

use crate::client::ForgejoClient;
use crate::mcp::types::Tool;

pub fn get_tools() -> Vec<Tool> {
    vec![
        // ==================== Repositories (12) ====================
        Tool {
            name: "search_repos".to_string(),
            description: "Search repositories by keyword, owner, or query parameters".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "keyword": { "type": "string", "description": "Search keyword" },
                    "q": { "type": "string", "description": "Search keyword alias" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 100 },
                    "sort": { "type": "string", "description": "Sort order", "default": "updated" },
                    "order": { "type": "string", "description": "Order direction (asc/desc)", "default": "desc" }
                },
                "required": []
            }),
        },
        Tool {
            name: "list_my_repos".to_string(),
            description: "List repositories belonging to the authenticated user".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 100 }
                },
                "required": []
            }),
        },
        Tool {
            name: "get_repo".to_string(),
            description: "Get repository metadata".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "get_file_content".to_string(),
            description: "Get file content as plain text or metadata. Optional line range start_line and end_line.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "filePath": { "type": "string", "description": "File path" },
                    "ref": { "type": "string", "description": "Branch, tag, or commit ref" },
                    "start_line": { "type": "number", "description": "Optional 1-indexed first line (inclusive)" },
                    "end_line": { "type": "number", "description": "Optional 1-indexed last line (inclusive)" },
                    "with_metadata": { "type": "boolean", "description": "Return full ContentsResponse JSON instead of plain text" }
                },
                "required": ["owner", "repo", "filePath"]
            }),
        },
        Tool {
            name: "create_file".to_string(),
            description: "Create a file with commit message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "filePath": { "type": "string", "description": "File path" },
                    "content": { "type": "string", "description": "File content (plain text, automatically base64-encoded)" },
                    "message": { "type": "string", "description": "Commit message" },
                    "branch_name": { "type": "string", "description": "Target branch name" },
                    "new_branch_name": { "type": "string", "description": "Optional new branch name" }
                },
                "required": ["owner", "repo", "filePath", "content", "message", "branch_name"]
            }),
        },
        Tool {
            name: "update_file".to_string(),
            description: "Update an existing file with commit message and SHA".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "filePath": { "type": "string", "description": "File path" },
                    "content": { "type": "string", "description": "File content (plain text, automatically base64-encoded)" },
                    "message": { "type": "string", "description": "Commit message" },
                    "sha": { "type": "string", "description": "SHA of the file being replaced" },
                    "branch_name": { "type": "string", "description": "Target branch name" },
                    "new_branch_name": { "type": "string", "description": "Optional new branch name" }
                },
                "required": ["owner", "repo", "filePath", "content", "message", "sha", "branch_name"]
            }),
        },
        Tool {
            name: "delete_file".to_string(),
            description: "Delete a file from repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "filePath": { "type": "string", "description": "File path" },
                    "message": { "type": "string", "description": "Commit message" },
                    "sha": { "type": "string", "description": "SHA of the file to delete" },
                    "branch_name": { "type": "string", "description": "Branch name" },
                    "new_branch_name": { "type": "string", "description": "Optional new branch name" }
                },
                "required": ["owner", "repo", "filePath", "message", "sha", "branch_name"]
            }),
        },
        Tool {
            name: "get_repo_tree".to_string(),
            description: "Get Git tree of repository (recursive or flat)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "ref": { "type": "string", "description": "Branch, tag, or commit SHA (defaults to default branch / HEAD)" },
                    "recursive": { "type": "boolean", "description": "Recursively get all subtrees" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 1000 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "list_repo_commits".to_string(),
            description: "List repository commits on branch or ref".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "sha": { "type": "string", "description": "SHA or branch to start from" },
                    "path": { "type": "string", "description": "Filter by file/directory path" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 100 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "list_branches".to_string(),
            description: "List branches in repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 100 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "create_branch".to_string(),
            description: "Create a new branch from reference".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "branch": { "type": "string", "description": "New branch name" },
                    "old_branch": { "type": "string", "description": "Source branch or ref" }
                },
                "required": ["owner", "repo", "branch", "old_branch"]
            }),
        },
        Tool {
            name: "delete_branch".to_string(),
            description: "Delete a branch from repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "branch": { "type": "string", "description": "Branch name to delete" }
                },
                "required": ["owner", "repo", "branch"]
            }),
        },

        // ==================== Issues & Milestones (12) ====================
        Tool {
            name: "search_issues".to_string(),
            description: "Query issues across repositories belonging to owner".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository or org owner" },
                    "q": { "type": "string", "description": "Search keyword" },
                    "state": { "type": "string", "description": "State (open|closed|all)", "default": "open" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 20 },
                    "labels": { "type": "string", "description": "Comma-separated labels" },
                    "milestones": { "type": "string", "description": "Comma-separated milestones" },
                    "type": { "type": "string", "description": "Type (issues|pulls)" },
                    "created_by": { "type": "string", "description": "Filter by creator username" },
                    "assigned_by": { "type": "string", "description": "Filter by assignee username" },
                    "mentioned_by": { "type": "string", "description": "Filter by mentioned username" }
                },
                "required": []
            }),
        },
        Tool {
            name: "list_repo_issues".to_string(),
            description: "List issues for repository filtered by state and labels".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "state": { "type": "string", "description": "State (open|closed|all)", "default": "open" },
                    "labels": { "type": "string", "description": "Comma-separated labels" },
                    "milestones": { "type": "string", "description": "Comma-separated milestones" },
                    "type": { "type": "string", "description": "Type (issues|pulls)" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 20 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "get_issue_by_index".to_string(),
            description: "Get issue details by repository index".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Issue index number" }
                },
                "required": ["owner", "repo", "index"]
            }),
        },
        Tool {
            name: "create_issue".to_string(),
            description: "Create new issue in repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "title": { "type": "string", "description": "Issue title" },
                    "body": { "type": "string", "description": "Issue description body" },
                    "assignee": { "type": "string", "description": "Assignee username" },
                    "assignees": { "type": "array", "items": { "type": "string" }, "description": "List of assignee usernames" },
                    "milestone": { "type": "number", "description": "Milestone ID" },
                    "labels": { "type": "array", "items": { "type": "number" }, "description": "List of label IDs" }
                },
                "required": ["owner", "repo", "title"]
            }),
        },
        Tool {
            name: "update_issue".to_string(),
            description: "Update issue title, body, assignee, or milestone".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Issue index" },
                    "title": { "type": "string", "description": "New title" },
                    "body": { "type": "string", "description": "New body content" },
                    "assignee": { "type": "string", "description": "Assignee username" },
                    "assignees": { "type": "string", "description": "Comma-separated assignee usernames" },
                    "milestone": { "type": "number", "description": "Milestone ID" }
                },
                "required": ["owner", "repo", "index"]
            }),
        },
        Tool {
            name: "issue_state_change".to_string(),
            description: "Change issue state to open or closed".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Issue index" },
                    "state": { "type": "string", "description": "New state (open|closed)" }
                },
                "required": ["owner", "repo", "index", "state"]
            }),
        },
        Tool {
            name: "list_issue_comments".to_string(),
            description: "List comments on an issue or pull request".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Issue/PR index" },
                    "since": { "type": "string", "description": "Filter comments after timestamp (RFC3339)" },
                    "before": { "type": "string", "description": "Filter comments before timestamp (RFC3339)" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 20 }
                },
                "required": ["owner", "repo", "index"]
            }),
        },
        Tool {
            name: "create_issue_comment".to_string(),
            description: "Add a comment to an issue or pull request".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Issue/PR index" },
                    "body": { "type": "string", "description": "Comment content" }
                },
                "required": ["owner", "repo", "index", "body"]
            }),
        },
        Tool {
            name: "list_repo_labels".to_string(),
            description: "List labels available in repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 100 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "add_issue_labels".to_string(),
            description: "Add labels to an issue".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Issue index" },
                    "labels": { "description": "Labels to add (comma-separated IDs or names, or array)" }
                },
                "required": ["owner", "repo", "index", "labels"]
            }),
        },
        Tool {
            name: "remove_issue_labels".to_string(),
            description: "Remove labels from an issue".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Issue index" },
                    "labels": { "type": "string", "description": "Comma-separated label IDs to remove (or omit to clear all)" }
                },
                "required": ["owner", "repo", "index"]
            }),
        },
        Tool {
            name: "list_repo_milestones".to_string(),
            description: "List milestones in repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "state": { "type": "string", "description": "State (open|closed|all)", "default": "open" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 100 }
                },
                "required": ["owner", "repo"]
            }),
        },

        // ==================== Pull Requests (6) ====================
        Tool {
            name: "list_repo_pull_requests".to_string(),
            description: "List pull requests in repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "state": { "type": "string", "description": "State (open|closed|all)", "default": "open" },
                    "sort": { "type": "string", "description": "Sort order (oldest|recentupdate|leastupdate|mostcomment)" },
                    "milestone": { "type": "string", "description": "Milestone ID or name" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 20 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "get_pull_request_by_index".to_string(),
            description: "Get pull request metadata by index".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Pull request index" }
                },
                "required": ["owner", "repo", "index"]
            }),
        },
        Tool {
            name: "get_pull_request_diff".to_string(),
            description: "Get raw unified diff or patch for pull request. Optional file_path filters to a single file.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "Pull request index" },
                    "file_path": { "type": "string", "description": "Optional file path to isolate hunks" }
                },
                "required": ["owner", "repo", "index"]
            }),
        },
        Tool {
            name: "create_pull_request".to_string(),
            description: "Open a pull request between head and base branches".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "head": { "type": "string", "description": "Head branch" },
                    "base": { "type": "string", "description": "Base branch" },
                    "title": { "type": "string", "description": "PR title" },
                    "body": { "type": "string", "description": "PR description" }
                },
                "required": ["owner", "repo", "head", "base", "title"]
            }),
        },
        Tool {
            name: "update_pull_request".to_string(),
            description: "Update pull request properties (title, body, base, assignee, milestone)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "PR index" },
                    "title": { "type": "string", "description": "New PR title" },
                    "body": { "type": "string", "description": "New PR description" },
                    "base": { "type": "string", "description": "New base branch" },
                    "assignee": { "type": "string", "description": "Assignee username" },
                    "milestone": { "type": "number", "description": "Milestone ID" }
                },
                "required": ["owner", "repo", "index"]
            }),
        },
        Tool {
            name: "merge_pull_request".to_string(),
            description: "Merge a pull request (merge, squash, rebase, rebase-merge)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "index": { "type": "number", "description": "PR index" },
                    "style": { "type": "string", "description": "Merge style: merge, rebase, rebase-merge, squash" },
                    "title": { "type": "string", "description": "Merge commit title" },
                    "message": { "type": "string", "description": "Merge commit message" },
                    "delete_branch_after_merge": { "type": "boolean", "description": "Delete head branch after merge" },
                    "force_merge": { "type": "boolean", "description": "Force merge regardless of status checks" },
                    "merge_when_checks_succeed": { "type": "boolean", "description": "Schedule merge when checks succeed" }
                },
                "required": ["owner", "repo", "index", "style"]
            }),
        },

        // ==================== Releases & Actions (6) ====================
        Tool {
            name: "list_releases".to_string(),
            description: "List releases for repository (state: all|draft|prerelease|published)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "state": { "type": "string", "description": "State filter (all|draft|prerelease|published)", "default": "all" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 20 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "get_release_by_tag".to_string(),
            description: "Retrieve a release by tag name".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "tag": { "type": "string", "description": "Release tag name" }
                },
                "required": ["owner", "repo", "tag"]
            }),
        },
        Tool {
            name: "create_release".to_string(),
            description: "Create a new release and optionally tag".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "tag_name": { "type": "string", "description": "Tag name for the release" },
                    "target_commitish": { "type": "string", "description": "Branch or commit SHA the tag points at" },
                    "name": { "type": "string", "description": "Release title" },
                    "body": { "type": "string", "description": "Release notes" },
                    "draft": { "type": "boolean", "description": "Draft release" },
                    "prerelease": { "type": "boolean", "description": "Pre-release" }
                },
                "required": ["owner", "repo", "tag_name"]
            }),
        },
        Tool {
            name: "list_workflow_runs".to_string(),
            description: "List Forgejo Actions workflow runs for repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "event": { "type": "string", "description": "Filter by event type (push, pull_request, etc.)" },
                    "status": { "type": "string", "description": "Filter by status (waiting, running, success, failure, cancelled)" },
                    "head_sha": { "type": "string", "description": "Filter by HEAD SHA" },
                    "run_number": { "type": "number", "description": "Filter by run number" },
                    "page": { "type": "number", "description": "Page number (1-based)", "default": 1 },
                    "limit": { "type": "number", "description": "Page size", "default": 30 }
                },
                "required": ["owner", "repo"]
            }),
        },
        Tool {
            name: "get_workflow_run".to_string(),
            description: "Get details and status of a specific workflow run".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "run_id": { "type": "number", "description": "Workflow run ID" }
                },
                "required": ["owner", "repo", "run_id"]
            }),
        },
        Tool {
            name: "dispatch_workflow".to_string(),
            description: "Trigger a workflow dispatch event".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Repository owner" },
                    "repo": { "type": "string", "description": "Repository name" },
                    "workflow": { "type": "string", "description": "Workflow filename or ID (e.g. image.yml)" },
                    "ref": { "type": "string", "description": "Git ref (branch, tag, or commit)" },
                    "inputs": { "description": "Workflow inputs (JSON object or string)" }
                },
                "required": ["owner", "repo", "workflow", "ref"]
            }),
        },
    ]
}

// ==================== Parameter Extraction Helpers ====================

fn get_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

fn get_required_str(args: &Value, key: &str) -> Result<String, String> {
    get_str(args, key)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing required argument: {}", key))
}

fn get_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_i64().filter(|i| *i >= 0).map(|i| i as u64))
            .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
    })
}

fn get_required_u64(args: &Value, key: &str) -> Result<u64, String> {
    get_u64(args, key).ok_or_else(|| format!("Missing required argument: {}", key))
}

fn get_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| {
        v.as_bool()
            .or_else(|| v.as_str().map(|s| s.eq_ignore_ascii_case("true")))
    })
}

// ==================== Tool Dispatcher ====================

pub async fn execute_tool(
    client: &ForgejoClient,
    name: &str,
    arguments: &Value,
) -> Result<String, String> {
    debug!("Executing tool: {} with args: {:?}", name, arguments);

    match name {
        // ----- Repositories -----
        "search_repos" => {
            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(q) = get_str(arguments, "q").or_else(|| get_str(arguments, "keyword")) {
                query_params.push(("q", q.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(100);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            if let Some(sort) = get_str(arguments, "sort") {
                query_params.push(("sort", sort.to_string()));
            }
            if let Some(order) = get_str(arguments, "order") {
                query_params.push(("order", order.to_string()));
            }

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            client.get("/repos/search", &q_refs).await
        }

        "list_my_repos" => {
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(100);
            let p_str = page.to_string();
            let l_str = limit.to_string();
            let query = [("page", p_str.as_str()), ("limit", l_str.as_str())];
            client.get("/user/repos", &query).await
        }

        "get_repo" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let path = format!("/repos/{}/{}", owner, repo);
            client.get(&path, &[]).await
        }

        "get_file_content" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let file_path = get_str(arguments, "filePath")
                .or_else(|| get_str(arguments, "file_path"))
                .or_else(|| get_str(arguments, "filepath"))
                .ok_or_else(|| "Missing required argument: filePath".to_string())?;

            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(r) = get_str(arguments, "ref") {
                query_params.push(("ref", r.to_string()));
            }
            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();

            let clean_path = file_path.trim_start_matches('/');
            let api_path = format!("/repos/{}/{}/contents/{}", owner, repo, clean_path);
            let raw_res = client.get(&api_path, &q_refs).await?;

            let with_metadata = get_bool(arguments, "with_metadata").unwrap_or(false);
            if with_metadata {
                return Ok(raw_res);
            }

            // Decode base64 content
            let parsed: Value = serde_json::from_str(&raw_res)
                .map_err(|e| format!("Failed to parse Forgejo contents JSON: {}", e))?;

            if let Some(b64_content) = parsed.get("content").and_then(|v| v.as_str()) {
                let clean_b64: String = b64_content.chars().filter(|c| !c.is_whitespace()).collect();
                let decoded = BASE64_STANDARD
                    .decode(&clean_b64)
                    .map_err(|e| format!("Base64 decode failed: {}", e))?;
                let mut text = String::from_utf8_lossy(&decoded).to_string();

                let start_line = get_u64(arguments, "start_line").map(|v| v as usize);
                let end_line = get_u64(arguments, "end_line").map(|v| v as usize);

                if start_line.is_some() || end_line.is_some() {
                    let lines: Vec<&str> = text.lines().collect();
                    let total = lines.len();
                    if total > 0 {
                        let start = start_line.unwrap_or(1).max(1);
                        let end = end_line.unwrap_or(total).min(total);
                        if start <= total && start <= end {
                            text = lines[(start - 1)..end].join("\n");
                        } else {
                            text = String::new();
                        }
                    }
                }
                Ok(text)
            } else {
                Ok(raw_res)
            }
        }

        "create_file" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let file_path = get_str(arguments, "filePath")
                .or_else(|| get_str(arguments, "file_path"))
                .ok_or_else(|| "Missing required argument: filePath".to_string())?;
            let content = get_required_str(arguments, "content")?;
            let message = get_required_str(arguments, "message")?;
            let branch = get_str(arguments, "branch_name")
                .or_else(|| get_str(arguments, "branch"))
                .ok_or_else(|| "Missing required argument: branch_name".to_string())?;

            let b64_content = BASE64_STANDARD.encode(content.as_bytes());
            let mut body = json!({
                "content": b64_content,
                "message": message,
                "branch": branch
            });

            if let Some(new_branch) = get_str(arguments, "new_branch_name")
                .or_else(|| get_str(arguments, "new_branch"))
            {
                body["new_branch"] = json!(new_branch);
            }

            let clean_path = file_path.trim_start_matches('/');
            let api_path = format!("/repos/{}/{}/contents/{}", owner, repo, clean_path);
            client.post(&api_path, &body).await
        }

        "update_file" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let file_path = get_str(arguments, "filePath")
                .or_else(|| get_str(arguments, "file_path"))
                .ok_or_else(|| "Missing required argument: filePath".to_string())?;
            let content = get_required_str(arguments, "content")?;
            let message = get_required_str(arguments, "message")?;
            let sha = get_required_str(arguments, "sha")?;
            let branch = get_str(arguments, "branch_name")
                .or_else(|| get_str(arguments, "branch"))
                .ok_or_else(|| "Missing required argument: branch_name".to_string())?;

            let b64_content = BASE64_STANDARD.encode(content.as_bytes());
            let mut body = json!({
                "content": b64_content,
                "message": message,
                "sha": sha,
                "branch": branch
            });

            if let Some(new_branch) = get_str(arguments, "new_branch_name")
                .or_else(|| get_str(arguments, "new_branch"))
            {
                body["new_branch"] = json!(new_branch);
            }

            let clean_path = file_path.trim_start_matches('/');
            let api_path = format!("/repos/{}/{}/contents/{}", owner, repo, clean_path);
            client.put(&api_path, &body).await
        }

        "delete_file" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let file_path = get_str(arguments, "filePath")
                .or_else(|| get_str(arguments, "file_path"))
                .ok_or_else(|| "Missing required argument: filePath".to_string())?;
            let message = get_required_str(arguments, "message")?;
            let sha = get_required_str(arguments, "sha")?;
            let branch = get_str(arguments, "branch_name")
                .or_else(|| get_str(arguments, "branch"))
                .ok_or_else(|| "Missing required argument: branch_name".to_string())?;

            let mut body = json!({
                "message": message,
                "sha": sha,
                "branch": branch
            });

            if let Some(new_branch) = get_str(arguments, "new_branch_name")
                .or_else(|| get_str(arguments, "new_branch"))
            {
                body["new_branch"] = json!(new_branch);
            }

            let clean_path = file_path.trim_start_matches('/');
            let api_path = format!("/repos/{}/{}/contents/{}", owner, repo, clean_path);
            client.delete_with_body(&api_path, &body).await
        }

        "get_repo_tree" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let ref_name = get_str(arguments, "ref").unwrap_or("HEAD");

            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(rec) = get_bool(arguments, "recursive") {
                query_params.push(("recursive", rec.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(1000);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            let path = format!("/repos/{}/{}/git/trees/{}", owner, repo, ref_name);
            client.get(&path, &q_refs).await
        }

        "list_repo_commits" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;

            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(sha) = get_str(arguments, "sha") {
                query_params.push(("sha", sha.to_string()));
            }
            if let Some(path) = get_str(arguments, "path") {
                query_params.push(("path", path.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(100);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            let path = format!("/repos/{}/{}/commits", owner, repo);
            client.get(&path, &q_refs).await
        }

        "list_branches" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(100);
            let p_str = page.to_string();
            let l_str = limit.to_string();
            let query = [("page", p_str.as_str()), ("limit", l_str.as_str())];
            let path = format!("/repos/{}/{}/branches", owner, repo);
            client.get(&path, &query).await
        }

        "create_branch" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let branch = get_str(arguments, "branch")
                .or_else(|| get_str(arguments, "new_branch_name"))
                .ok_or_else(|| "Missing required argument: branch".to_string())?;
            let old_branch = get_str(arguments, "old_branch")
                .or_else(|| get_str(arguments, "old_branch_name"))
                .or_else(|| get_str(arguments, "old_ref_name"))
                .ok_or_else(|| "Missing required argument: old_branch".to_string())?;

            let body = json!({
                "new_branch_name": branch,
                "old_branch_name": old_branch
            });
            let path = format!("/repos/{}/{}/branches", owner, repo);
            client.post(&path, &body).await
        }

        "delete_branch" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let branch = get_required_str(arguments, "branch")?;
            let path = format!("/repos/{}/{}/branches/{}", owner, repo, branch);
            client.delete(&path).await
        }

        // ----- Issues & Milestones -----
        "search_issues" => {
            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(owner) = get_str(arguments, "owner") {
                query_params.push(("owner", owner.to_string()));
            }
            if let Some(q) = get_str(arguments, "q") {
                query_params.push(("q", q.to_string()));
            }
            if let Some(state) = get_str(arguments, "state") {
                query_params.push(("state", state.to_string()));
            }
            if let Some(labels) = get_str(arguments, "labels") {
                query_params.push(("labels", labels.to_string()));
            }
            if let Some(milestones) = get_str(arguments, "milestones") {
                query_params.push(("milestones", milestones.to_string()));
            }
            if let Some(t) = get_str(arguments, "type") {
                query_params.push(("type", t.to_string()));
            }
            if let Some(c) = get_str(arguments, "created_by") {
                query_params.push(("created_by", c.to_string()));
            }
            if let Some(a) = get_str(arguments, "assigned_by") {
                query_params.push(("assigned_by", a.to_string()));
            }
            if let Some(m) = get_str(arguments, "mentioned_by") {
                query_params.push(("mentioned_by", m.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(20);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            client.get("/repos/issues/search", &q_refs).await
        }

        "list_repo_issues" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;

            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(state) = get_str(arguments, "state") {
                query_params.push(("state", state.to_string()));
            }
            if let Some(labels) = get_str(arguments, "labels") {
                query_params.push(("labels", labels.to_string()));
            }
            if let Some(milestones) = get_str(arguments, "milestones") {
                query_params.push(("milestones", milestones.to_string()));
            }
            if let Some(t) = get_str(arguments, "type") {
                query_params.push(("type", t.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(20);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            let path = format!("/repos/{}/{}/issues", owner, repo);
            client.get(&path, &q_refs).await
        }

        "get_issue_by_index" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;
            let path = format!("/repos/{}/{}/issues/{}", owner, repo, index);
            client.get(&path, &[]).await
        }

        "create_issue" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let title = get_required_str(arguments, "title")?;

            let mut body = json!({ "title": title });
            if let Some(b) = get_str(arguments, "body") {
                body["body"] = json!(b);
            }
            if let Some(a) = get_str(arguments, "assignee") {
                body["assignee"] = json!(a);
            }
            if let Some(arr) = arguments.get("assignees").and_then(|v| v.as_array()) {
                body["assignees"] = json!(arr);
            }
            if let Some(m) = get_u64(arguments, "milestone") {
                body["milestone"] = json!(m);
            }
            if let Some(l) = arguments.get("labels") {
                body["labels"] = json!(l);
            }

            let path = format!("/repos/{}/{}/issues", owner, repo);
            client.post(&path, &body).await
        }

        "update_issue" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;

            let mut body = json!({});
            if let Some(t) = get_str(arguments, "title") {
                body["title"] = json!(t);
            }
            if let Some(b) = get_str(arguments, "body") {
                body["body"] = json!(b);
            }
            if let Some(a) = get_str(arguments, "assignee") {
                body["assignee"] = json!(a);
            }
            if let Some(arr) = arguments.get("assignees") {
                if let Some(s) = arr.as_str() {
                    let list: Vec<&str> = s.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()).collect();
                    body["assignees"] = json!(list);
                } else {
                    body["assignees"] = json!(arr);
                }
            }
            if let Some(m) = get_u64(arguments, "milestone") {
                body["milestone"] = json!(m);
            }

            let path = format!("/repos/{}/{}/issues/{}", owner, repo, index);
            client.patch(&path, &body).await
        }

        "issue_state_change" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;
            let state = get_required_str(arguments, "state")?;

            let body = json!({ "state": state });
            let path = format!("/repos/{}/{}/issues/{}", owner, repo, index);
            client.patch(&path, &body).await
        }

        "list_issue_comments" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;

            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(since) = get_str(arguments, "since") {
                query_params.push(("since", since.to_string()));
            }
            if let Some(before) = get_str(arguments, "before") {
                query_params.push(("before", before.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(20);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            let path = format!("/repos/{}/{}/issues/{}/comments", owner, repo, index);
            client.get(&path, &q_refs).await
        }

        "create_issue_comment" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;
            let body_text = get_required_str(arguments, "body")?;

            let body = json!({ "body": body_text });
            let path = format!("/repos/{}/{}/issues/{}/comments", owner, repo, index);
            client.post(&path, &body).await
        }

        "list_repo_labels" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(100);
            let p_str = page.to_string();
            let l_str = limit.to_string();
            let query = [("page", p_str.as_str()), ("limit", l_str.as_str())];
            let path = format!("/repos/{}/{}/labels", owner, repo);
            client.get(&path, &query).await
        }

        "add_issue_labels" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;

            let labels_val = arguments.get("labels").ok_or_else(|| "Missing required argument: labels".to_string())?;
            let labels_list: Vec<Value> = if let Some(s) = labels_val.as_str() {
                s.split(',')
                    .map(|x| x.trim())
                    .filter(|x| !x.is_empty())
                    .map(|x| {
                        if let Ok(num) = x.parse::<i64>() {
                            json!(num)
                        } else {
                            json!(x)
                        }
                    })
                    .collect()
            } else if let Some(arr) = labels_val.as_array() {
                arr.clone()
            } else {
                vec![labels_val.clone()]
            };

            let body = json!({ "labels": labels_list });
            let path = format!("/repos/{}/{}/issues/{}/labels", owner, repo, index);
            client.post(&path, &body).await
        }

        "remove_issue_labels" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;

            if let Some(s) = get_str(arguments, "labels") {
                let ids: Vec<&str> = s.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()).collect();
                if ids.is_empty() {
                    let path = format!("/repos/{}/{}/issues/{}/labels", owner, repo, index);
                    client.delete(&path).await
                } else {
                    let mut last_res = "{}".to_string();
                    for id in ids {
                        let path = format!("/repos/{}/{}/issues/{}/labels/{}", owner, repo, index, id);
                        last_res = client.delete(&path).await?;
                    }
                    Ok(last_res)
                }
            } else {
                let path = format!("/repos/{}/{}/issues/{}/labels", owner, repo, index);
                client.delete(&path).await
            }
        }

        "list_repo_milestones" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let state = get_str(arguments, "state").unwrap_or("open");
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(100);
            let p_str = page.to_string();
            let l_str = limit.to_string();
            let query = [("state", state), ("page", p_str.as_str()), ("limit", l_str.as_str())];
            let path = format!("/repos/{}/{}/milestones", owner, repo);
            client.get(&path, &query).await
        }

        // ----- Pull Requests -----
        "list_repo_pull_requests" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;

            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(state) = get_str(arguments, "state") {
                query_params.push(("state", state.to_string()));
            }
            if let Some(sort) = get_str(arguments, "sort") {
                query_params.push(("sort", sort.to_string()));
            }
            if let Some(milestone) = get_str(arguments, "milestone") {
                query_params.push(("milestone", milestone.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(20);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            let path = format!("/repos/{}/{}/pulls", owner, repo);
            client.get(&path, &q_refs).await
        }

        "get_pull_request_by_index" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;
            let path = format!("/repos/{}/{}/pulls/{}", owner, repo, index);
            client.get(&path, &[]).await
        }

        "get_pull_request_diff" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;

            let path = format!("/repos/{}/{}/pulls/{}.diff", owner, repo, index);
            let diff = client.get_raw(&path, &[]).await?;

            if let Some(filter_file) = get_str(arguments, "file_path") {
                let clean_file = filter_file.trim_start_matches('/');
                let mut matched_sections = Vec::new();
                for section in diff.split("diff --git ") {
                    if section.trim().is_empty() {
                        continue;
                    }
                    let first_line = section.lines().next().unwrap_or("");
                    if first_line.contains(&format!("a/{}", clean_file))
                        || first_line.contains(&format!("b/{}", clean_file))
                        || first_line.contains(clean_file)
                    {
                        matched_sections.push(format!("diff --git {}", section));
                    }
                }
                if matched_sections.is_empty() {
                    Ok(format!("No diff found for file: {}", clean_file))
                } else {
                    Ok(matched_sections.join(""))
                }
            } else {
                Ok(diff)
            }
        }

        "create_pull_request" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let head = get_required_str(arguments, "head")?;
            let base = get_required_str(arguments, "base")?;
            let title = get_required_str(arguments, "title")?;

            let mut body = json!({
                "head": head,
                "base": base,
                "title": title
            });
            if let Some(b) = get_str(arguments, "body") {
                body["body"] = json!(b);
            }

            let path = format!("/repos/{}/{}/pulls", owner, repo);
            client.post(&path, &body).await
        }

        "update_pull_request" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;

            let mut body = json!({});
            if let Some(t) = get_str(arguments, "title") {
                body["title"] = json!(t);
            }
            if let Some(b) = get_str(arguments, "body") {
                body["body"] = json!(b);
            }
            if let Some(base) = get_str(arguments, "base") {
                body["base"] = json!(base);
            }
            if let Some(a) = get_str(arguments, "assignee") {
                body["assignee"] = json!(a);
            }
            if let Some(m) = get_u64(arguments, "milestone") {
                body["milestone"] = json!(m);
            }

            let path = format!("/repos/{}/{}/pulls/{}", owner, repo, index);
            client.patch(&path, &body).await
        }

        "merge_pull_request" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let index = get_required_u64(arguments, "index")?;
            let style = get_required_str(arguments, "style")?;

            let mut body = json!({
                "Do": style,
                "style": style
            });
            if let Some(t) = get_str(arguments, "title") {
                body["MergeTitleField"] = json!(t);
            }
            if let Some(m) = get_str(arguments, "message") {
                body["MergeMessageField"] = json!(m);
            }
            if let Some(d) = get_bool(arguments, "delete_branch_after_merge") {
                body["delete_branch_after_merge"] = json!(d);
            }
            if let Some(f) = get_bool(arguments, "force_merge") {
                body["force_merge"] = json!(f);
            }
            if let Some(w) = get_bool(arguments, "merge_when_checks_succeed") {
                body["merge_when_checks_succeed"] = json!(w);
            }

            let path = format!("/repos/{}/{}/pulls/{}/merge", owner, repo, index);
            client.post(&path, &body).await
        }

        // ----- Releases & Actions -----
        "list_releases" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(20);
            let p_str = page.to_string();
            let l_str = limit.to_string();
            let query = [("page", p_str.as_str()), ("limit", l_str.as_str())];

            let path = format!("/repos/{}/{}/releases", owner, repo);
            let res = client.get(&path, &query).await?;

            let state_filter = get_str(arguments, "state").unwrap_or("all");
            if state_filter != "all" {
                if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(&res) {
                    let filtered: Vec<Value> = items
                        .into_iter()
                        .filter(|item| {
                            let is_draft = item.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
                            let is_prerelease = item.get("prerelease").and_then(|v| v.as_bool()).unwrap_or(false);
                            match state_filter {
                                "draft" => is_draft,
                                "prerelease" => is_prerelease,
                                "published" => !is_draft && !is_prerelease,
                                _ => true,
                            }
                        })
                        .collect();
                    return serde_json::to_string(&filtered)
                        .map_err(|e| format!("Failed to serialize filtered releases: {}", e));
                }
            }
            Ok(res)
        }

        "get_release_by_tag" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let tag = get_required_str(arguments, "tag")?;
            let path = format!("/repos/{}/{}/releases/tags/{}", owner, repo, tag);
            client.get(&path, &[]).await
        }

        "create_release" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let tag_name = get_required_str(arguments, "tag_name")?;

            let mut body = json!({ "tag_name": tag_name });
            if let Some(tc) = get_str(arguments, "target_commitish") {
                body["target_commitish"] = json!(tc);
            }
            if let Some(name) = get_str(arguments, "name") {
                body["name"] = json!(name);
            }
            if let Some(body_text) = get_str(arguments, "body") {
                body["body"] = json!(body_text);
            }
            if let Some(d) = get_bool(arguments, "draft") {
                body["draft"] = json!(d);
            }
            if let Some(p) = get_bool(arguments, "prerelease") {
                body["prerelease"] = json!(p);
            }

            let path = format!("/repos/{}/{}/releases", owner, repo);
            client.post(&path, &body).await
        }

        "list_workflow_runs" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;

            let mut query_params: Vec<(&str, String)> = Vec::new();
            if let Some(event) = get_str(arguments, "event") {
                query_params.push(("event", event.to_string()));
            }
            if let Some(status) = get_str(arguments, "status") {
                query_params.push(("status", status.to_string()));
            }
            if let Some(head_sha) = get_str(arguments, "head_sha") {
                query_params.push(("head_sha", head_sha.to_string()));
            }
            if let Some(run_num) = get_u64(arguments, "run_number") {
                query_params.push(("run_number", run_num.to_string()));
            }
            let page = get_u64(arguments, "page").unwrap_or(1);
            let limit = get_u64(arguments, "limit").unwrap_or(30);
            query_params.push(("page", page.to_string()));
            query_params.push(("limit", limit.to_string()));

            let q_refs: Vec<(&str, &str)> = query_params
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            let path = format!("/repos/{}/{}/actions/runs", owner, repo);
            client.get(&path, &q_refs).await
        }

        "get_workflow_run" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let run_id = get_required_u64(arguments, "run_id")?;
            let path = format!("/repos/{}/{}/actions/runs/{}", owner, repo, run_id);
            client.get(&path, &[]).await
        }

        "dispatch_workflow" => {
            let owner = get_required_str(arguments, "owner")?;
            let repo = get_required_str(arguments, "repo")?;
            let workflow = get_required_str(arguments, "workflow")?;
            let git_ref = get_required_str(arguments, "ref")?;

            let mut body = json!({ "ref": git_ref });
            if let Some(inputs) = arguments.get("inputs") {
                if let Some(s) = inputs.as_str() {
                    if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                        body["inputs"] = parsed;
                    } else {
                        body["inputs"] = json!(s);
                    }
                } else {
                    body["inputs"] = inputs.clone();
                }
            }

            let path = format!("/repos/{}/{}/actions/workflows/{}/dispatches", owner, repo, workflow);
            client.post(&path, &body).await
        }

        other => Err(format!("Unknown tool: {}", other)),
    }
}
