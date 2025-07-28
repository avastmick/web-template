#![allow(clippy::unwrap_used)]

//! Route coverage test
//!
//! This test ensures that all API endpoints defined in the server
//! have corresponding endpoint tests.

use regex::Regex;
use std::collections::HashSet;

/// Normalize a path by replacing UUIDs and other dynamic parts with placeholders
fn normalize_path(path: &str) -> String {
    // UUID pattern
    let uuid_regex =
        Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
            .unwrap();

    // Replace UUIDs with {id}
    let mut normalized = uuid_regex.replace_all(path, "{id}").to_string();

    // Replace other common patterns
    normalized = normalized.replace("{}", "{id}");

    // Handle specific test IDs
    if normalized.contains("/conversations/invalid-id-123") {
        normalized = normalized.replace("/conversations/invalid-id-123", "/conversations/{id}");
    }

    // Handle email patterns
    let email_regex = Regex::new(r"/invites/[^/]+@[^/]+\.[^/]+").unwrap();
    normalized = email_regex
        .replace(&normalized, "/invites/{email}")
        .to_string();

    // Handle other common ID patterns (numeric or alphanumeric)
    let segments: Vec<&str> = normalized.split('/').collect();
    let mut result_segments = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        if i > 0 && segments[i - 1] == "conversations" && *segment != "{id}" {
            // Replace any conversation ID with {id}
            result_segments.push("{id}");
        } else if i > 0
            && segments[i - 1] == "invites"
            && *segment != "{id}"
            && *segment != "{email}"
        {
            // Replace invite IDs
            result_segments.push("{id}");
        } else {
            result_segments.push(segment);
        }
    }

    result_segments.join("/")
}

/// Extract all API routes from the routes.rs file
fn extract_api_routes() -> HashSet<(String, String)> {
    let routes_content = include_str!("../../src/routes.rs");
    let mut routes = HashSet::new();

    // Regex to match route definitions
    // Matches patterns like: .route("/api/...", get(...))
    let route_regex = Regex::new(r#"\.route\("(/api/[^"]+)",\s*(?:get|post|put|delete|axum::routing::(?:get|post|put|delete))\("#).unwrap();

    for cap in route_regex.captures_iter(routes_content) {
        let path = cap[1].to_string();

        // Extract method from the surrounding context
        let method = if cap[0].contains("get(") || cap[0].contains("::get(") {
            "GET"
        } else if cap[0].contains("post(") || cap[0].contains("::post(") {
            "POST"
        } else if cap[0].contains("put(") || cap[0].contains("::put(") {
            "PUT"
        } else if cap[0].contains("delete(") || cap[0].contains("::delete(") {
            "DELETE"
        } else {
            continue;
        };

        routes.insert((method.to_string(), path));
    }

    routes
}

/// Extract tested routes from endpoint test files
#[allow(clippy::too_many_lines)]
fn extract_tested_routes() -> HashSet<(String, String)> {
    let mut tested = HashSet::new();

    // List of test file contents
    let test_files = vec![
        include_str!("./auth_tests.rs"),
        include_str!("./payment_tests.rs"),
    ];

    // Regex to match test API calls
    let uri_regex = Regex::new(r#"(?:uri|&format!\()?"/api/[^"]+"#).unwrap();
    let method_regex = Regex::new(r"Method::(\w+)").unwrap();

    for content in test_files {
        let lines: Vec<&str> = content.lines().collect();

        // Find all API paths being tested
        for line in &lines {
            if let Some(uri_match) = uri_regex.find(line) {
                let uri = uri_match.as_str();
                let path = uri
                    .trim_start_matches("&format!(\"")
                    .trim_start_matches("uri|\"")
                    .trim_start_matches('"')
                    .trim_end_matches('"');

                // Look for method in nearby lines
                if let Some(method_match) = method_regex.find(line) {
                    let method = method_match.as_str().replace("Method::", "").to_uppercase();

                    // Handle parameterized paths
                    // Strip query parameters if present
                    let path_without_query = path.split('?').next().unwrap_or(path);

                    let normalized_path = normalize_path(path_without_query);
                    tested.insert((method, normalized_path));
                }
            }
        }

        // Also check for specific test patterns (including multi-line)
        for (i, line) in lines.iter().enumerate() {
            // Pattern: send_authenticated_request or send_json_request
            if line.contains("send_authenticated_json_request")
                || line.contains("send_authenticated_request")
                || line.contains("send_json_request")
            {
                // For multi-line calls, look for Method and path in the next few lines
                let mut method = None;
                let mut path = None;

                // Check current line first
                if let Some(method_match) = method_regex.find(line) {
                    method = Some(method_match.as_str().replace("Method::", "").to_uppercase());
                }
                if let Some(start) = line.find("\"/api/") {
                    if let Some(end) = line[start + 1..].find('"') {
                        path = Some(line[start + 1..start + 1 + end].to_string());
                    }
                }

                // If not found on same line, check next 5 lines
                for j in 1..=5 {
                    if i + j >= lines.len() {
                        break;
                    }
                    let next_line = lines[i + j];

                    if method.is_none() {
                        if let Some(method_match) = method_regex.find(next_line) {
                            method =
                                Some(method_match.as_str().replace("Method::", "").to_uppercase());
                        }
                    }

                    if path.is_none() {
                        if let Some(start) = next_line.find("\"/api/") {
                            if let Some(end) = next_line[start + 1..].find('"') {
                                path = Some(next_line[start + 1..start + 1 + end].to_string());
                            }
                        } else if next_line.contains('&')
                            && (next_line.contains("_uri") || next_line.contains("_url"))
                        {
                            // Handle variables like &poll_uri or &callback_uri
                            // Look back up to 10 lines for the format! definition
                            for k in 1..=10 {
                                if i < k {
                                    break;
                                }
                                let prev_line = lines[i - k];
                                if prev_line.contains("format!") && prev_line.contains("\"/api/") {
                                    if let Some(start) = prev_line.find("\"/api/") {
                                        // Extract path up to ? or closing quote
                                        let path_start = start + 1;
                                        let path_str = &prev_line[path_start..];
                                        let end = path_str.find('?').unwrap_or_else(|| {
                                            path_str.find('"').unwrap_or(path_str.len())
                                        });
                                        path = Some(path_str[..end].to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // If we found both, we can stop
                    if method.is_some() && path.is_some() {
                        break;
                    }
                }

                // If we found both method and path, add to tested routes
                if let (Some(method), Some(path)) = (method, path) {
                    // Strip query parameters if present
                    let path_without_query = path.split('?').next().unwrap_or(&path);

                    let normalized_path = normalize_path(path_without_query);
                    tested.insert((method, normalized_path));
                }
            }
        }
    }

    tested
}

/// Print route coverage summary
fn print_coverage_summary(
    api_routes: &HashSet<(String, String)>,
    tested_routes: &HashSet<(String, String)>,
    info_exempted: &HashSet<(String, String)>,
    warn_exempted: &HashSet<(String, String)>,
    untested_not_exempted: &[&(String, String)],
) {
    // Calculate actual untested routes
    let untested_routes: HashSet<_> = api_routes.difference(tested_routes).cloned().collect();

    // Routes that are in WARN list but actually have tests
    let warn_but_tested: HashSet<_> = warn_exempted.intersection(tested_routes).cloned().collect();

    // Routes that are in WARN list and don't have tests
    let warn_and_untested: HashSet<_> = warn_exempted
        .intersection(&untested_routes)
        .cloned()
        .collect();

    // Routes tested but not in api_routes (phantom tests)
    let phantom_tests: HashSet<_> = tested_routes.difference(api_routes).cloned().collect();

    println!("\n📊 Route Coverage Summary:");
    println!("   Total API routes defined: {}", api_routes.len());
    println!(
        "   Routes with tests: {}",
        tested_routes.intersection(api_routes).count()
    );
    println!("   Routes without tests: {}", untested_routes.len());

    if !phantom_tests.is_empty() {
        println!(
            "\n   ⚠️  Tests found for non-existent routes: {}",
            phantom_tests.len()
        );
        let mut phantom_list: Vec<_> = phantom_tests.iter().collect();
        phantom_list.sort();
        for (method, path) in phantom_list.iter().take(5) {
            println!("      - {method} {path}");
        }
        if phantom_list.len() > 5 {
            println!("      ... and {} more", phantom_list.len() - 5);
        }
    }

    println!("\n   Exemption details:");
    println!(
        "   - INFO exempted (don't need tests): {}",
        info_exempted.len()
    );
    println!(
        "   - WARN exempted (should have tests): {}",
        warn_exempted.len()
    );
    println!("     - Actually tested: {}", warn_but_tested.len());
    println!("     - Still need tests: {}", warn_and_untested.len());
    println!(
        "   - Untested and not exempted: {}",
        untested_not_exempted.len()
    );

    if untested_not_exempted.is_empty() && warn_and_untested.is_empty() {
        println!("\n✅ All API routes have endpoint tests!");
    } else if untested_not_exempted.is_empty() {
        println!(
            "\n⚠️  All required routes are tested, but {} routes in WARN list still need tests!",
            warn_and_untested.len()
        );
    }
}

#[test]
fn test_all_api_routes_have_tests() {
    let api_routes = extract_api_routes();
    let tested_routes = extract_tested_routes();

    // Find routes without tests
    let mut untested_routes: Vec<_> = api_routes.difference(&tested_routes).collect();
    untested_routes.sort();

    // INFO exemptions: endpoints that don't need tests (browser interaction, special setup)
    let info_exemptions = vec![
        // OAuth endpoints require browser interaction
        ("GET", "/api/auth/oauth/github"),
        ("GET", "/api/auth/oauth/google"),
        // Webhook endpoints might not need standard auth tests
        ("POST", "/api/webhooks/stripe"),
        // Debug endpoints
        ("GET", "/api/debug/error/{error_type}"),
        ("GET", "/api/debug/message"),
    ];

    // WARN exemptions: endpoints that MUST have tests but are temporarily exempted
    // NOTE: This list should only contain routes that don't have tests yet
    // Routes that have tests should be removed from this list
    let warn_exemptions = vec![
        // Admin endpoints (not yet implemented)
        ("GET", "/api/admin/invites"),
        ("POST", "/api/admin/invites"),
        ("DELETE", "/api/admin/invites/{id}"),
        // Other endpoints without tests
        ("GET", "/api/invites/{email}"),
        ("GET", "/api/auth/verify"),
        ("GET", "/api/ai/chat/stream"),
        ("GET", "/api/ai/conversations"),
        ("GET", "/api/ai/conversations/{id}"),
        ("GET", "/api/ai/sessions/{id}"),
        ("GET", "/api/ai/health"),
        ("GET", "/api/ai/info"),
        ("GET", "/api/ai/usage"),
        ("POST", "/api/ai/analyze/code"),
        ("POST", "/api/ai/chat"),
        ("POST", "/api/ai/chat/contextual"),
        ("POST", "/api/ai/moderate"),
        ("POST", "/api/ai/upload"),
    ];

    let info_exempted: HashSet<_> = info_exemptions
        .into_iter()
        .map(|(m, p)| (m.to_string(), p.to_string()))
        .collect();

    let warn_exempted: HashSet<_> = warn_exemptions
        .into_iter()
        .map(|(m, p)| (m.to_string(), p.to_string()))
        .collect();

    let all_exempted: HashSet<_> = info_exempted.union(&warn_exempted).cloned().collect();

    let untested_not_exempted: Vec<_> = untested_routes
        .into_iter()
        .filter(|(m, p)| !all_exempted.contains(&(m.to_string(), p.to_string())))
        .collect();

    if !untested_not_exempted.is_empty() {
        println!(
            "\n❌ ERROR: {} API routes without endpoint tests:",
            untested_not_exempted.len()
        );
        for (method, path) in &untested_not_exempted {
            println!("  ❌ {method} {path}");
        }
        println!("\n  These routes MUST have tests added immediately!");
        panic!("Found untested routes that are not in exemption lists");
    }

    // Show INFO exempted routes (don't need tests)
    if !info_exempted.is_empty() {
        println!(
            "\nℹ️  INFO Exempted routes ({}) - These don't require tests:",
            info_exempted.len()
        );
        let mut info_list: Vec<_> = info_exempted.iter().collect();
        info_list.sort();
        for (method, path) in info_list {
            println!("  ℹ️  {method} {path}");
        }
    }

    // Show WARN exempted routes (need tests but temporarily exempted)
    if !warn_exempted.is_empty() {
        println!(
            "\n⚠️  WARN Exempted routes ({}) - These MUST have tests added:",
            warn_exempted.len()
        );
        let mut warn_list: Vec<_> = warn_exempted.iter().collect();
        warn_list.sort();
        for (method, path) in warn_list {
            println!("  ⚠️  {method} {path}");
        }
    }

    // Always show summary
    print_coverage_summary(
        &api_routes,
        &tested_routes,
        &info_exempted,
        &warn_exempted,
        &untested_not_exempted,
    );
}
