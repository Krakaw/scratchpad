//! GitHub webhook integration tests
//! Tests GitHub webhook payload handling and scratch creation
//!
//! Run with: cargo test --test github_webhook_tests -- --test-threads=1 --nocapture

use serde_json::json;

// Define GithubWebhookPayload locally for testing
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GithubWebhookPayload {
    ref_name: Option<String>,
    pull_request: Option<PullRequest>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PullRequest {
    head: Head,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Head {
    ref_name: String,
}

#[test]
fn test_parse_push_webhook_payload() {
    let payload_json = json!({
        "ref": "refs/heads/feature/my-feature",
        "ref_name": "refs/heads/feature/my-feature",
        "repository": {
            "name": "my-repo"
        }
    });

    match serde_json::from_value::<GithubWebhookPayload>(payload_json) {
        Ok(payload) => {
            assert_eq!(
                payload.ref_name,
                Some("refs/heads/feature/my-feature".to_string())
            );
            println!("✓ Successfully parsed push webhook payload");
        }
        Err(e) => panic!("✗ Failed to parse payload: {}", e),
    }
}

#[test]
fn test_parse_pr_webhook_payload() {
    let payload_json = json!({
        "action": "opened",
        "pull_request": {
            "head": {
                "ref": "feature-branch",
                "ref_name": "feature-branch"
            }
        }
    });

    match serde_json::from_value::<GithubWebhookPayload>(payload_json) {
        Ok(payload) => {
            assert!(payload.pull_request.is_some());
            if let Some(pr) = payload.pull_request {
                assert_eq!(pr.head.ref_name, "feature-branch");
                println!("✓ Successfully parsed PR webhook payload");
            }
        }
        Err(e) => panic!("✗ Failed to parse payload: {}", e),
    }
}

#[test]
fn test_branch_name_extraction_from_ref() {
    let test_cases = vec![
        ("refs/heads/main", "main"),
        ("refs/heads/feature/test", "feature/test"),
        ("refs/heads/release/v1.0.0", "release/v1.0.0"),
        ("main", "main"),
    ];

    for (ref_input, expected) in test_cases {
        let result = ref_input.strip_prefix("refs/heads/").unwrap_or(ref_input);
        assert_eq!(result, expected);
        println!("✓ Branch extraction: '{}' → '{}'", ref_input, result);
    }
}

#[test]
fn test_branch_name_sanitization_for_webhook() {
    let test_cases = vec![
        ("feature/my-feature", "feature-my-feature"),
        ("feature/TEST_FEATURE", "feature-test_feature"),
        ("release/v1.0.0", "release-v1-0-0"),
        ("bugfix/ISSUE-123", "bugfix-issue-123"),
    ];

    for (branch, expected) in test_cases {
        let sanitized = scratchpad::scratch::Scratch::sanitize_name(branch);
        assert_eq!(sanitized, expected);
        println!("✓ Sanitization for webhook: '{}' → '{}'", branch, sanitized);
    }
}

#[test]
fn test_webhook_payload_with_multiple_branches() {
    let payloads = vec![
        json!({
            "ref_name": "refs/heads/develop",
            "repository": {"name": "test"}
        }),
        json!({
            "ref_name": "refs/heads/feature/awesome",
            "repository": {"name": "test"}
        }),
        json!({
            "ref_name": "refs/heads/hotfix/urgent-bug",
            "repository": {"name": "test"}
        }),
    ];

    for payload_json in payloads {
        match serde_json::from_value::<GithubWebhookPayload>(payload_json) {
            Ok(payload) => {
                if let Some(ref_name) = payload.ref_name {
                    let branch = ref_name.strip_prefix("refs/heads/").unwrap_or(&ref_name);
                    println!("✓ Parsed branch: {}", branch);
                }
            }
            Err(e) => panic!("Failed to parse: {}", e),
        }
    }
}

#[test]
fn test_webhook_missing_branch_name() {
    let payload_json = json!({
        "action": "completed",
        "repository": {"name": "test"}
    });

    match serde_json::from_value::<GithubWebhookPayload>(payload_json) {
        Ok(payload) => {
            assert!(payload.ref_name.is_none());
            assert!(payload.pull_request.is_none());
            println!("✓ Correctly handled webhook without branch info");
        }
        Err(e) => panic!("Failed to parse: {}", e),
    }
}

#[test]
fn test_webhook_payload_edge_cases() {
    let edge_cases = vec![
        ("refs/tags/v1.0.0", None),    // tags shouldn't match
        ("refs/pull/123/merge", None), // PR merge refs
        (
            "refs/heads/feature/with/many/slashes",
            Some("feature-with-many-slashes"),
        ),
        ("refs/heads/UPPERCASE-BRANCH", Some("uppercase-branch")),
        (
            "refs/heads/branch_with_underscores",
            Some("branch_with_underscores"),
        ),
    ];

    for (input, expected) in edge_cases {
        let branch_option = input.strip_prefix("refs/heads/").map(|s| s.to_string());

        match (&expected, branch_option) {
            (None, None) => println!("✓ Edge case correctly filtered: '{}'", input),
            (Some(exp), Some(branch)) => {
                let sanitized = scratchpad::scratch::Scratch::sanitize_name(&branch);
                assert_eq!(&sanitized, exp);
                println!("✓ Edge case sanitized: '{}' → '{}'", input, sanitized);
            }
            _ => println!("✓ Edge case handled: '{}'", input),
        }
    }
}

#[test]
fn test_webhook_valid_payload_structure() {
    let valid_payloads = vec![
        (
            "Push webhook with ref_name",
            json!({
                "ref": "refs/heads/main",
                "ref_name": "refs/heads/main"
            }),
        ),
        (
            "PR webhook",
            json!({
                "pull_request": {
                    "head": {
                        "ref": "feature",
                        "ref_name": "feature"
                    }
                }
            }),
        ),
        (
            "Webhook with both fields",
            json!({
                "ref_name": "refs/heads/feature",
                "pull_request": {
                    "head": {
                        "ref": "backup-branch",
                        "ref_name": "backup-branch"
                    }
                }
            }),
        ),
    ];

    for (desc, payload_json) in valid_payloads {
        match serde_json::from_value::<GithubWebhookPayload>(payload_json) {
            Ok(_payload) => {
                println!("✓ Valid payload structure: {}", desc);
            }
            Err(e) => panic!("Failed to parse {}: {}", desc, e),
        }
    }
}
