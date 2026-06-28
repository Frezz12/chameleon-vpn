use regex::Regex;
use serde_json::{json, Value};

use crate::geo_db::GeoDatabase;

/// Evaluate a domain against all rules and return which node it would route through
pub async fn evaluate_domain(domain: &str, rules: &[Value], geo_db: Option<&GeoDatabase>) -> Result<Value, String> {
    let mut matched_rules: Vec<(&Value, i64)> = Vec::new();

    for rule in rules {
        let enabled = rule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        if !enabled {
            continue;
        }

        let rule_type = rule.get("rule_type").and_then(|v| v.as_str()).unwrap_or("");
        let value = rule.get("value").and_then(|v| v.as_str()).unwrap_or("");
        let priority = rule.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);

        if value.is_empty() {
            continue;
        }

        let matched = match rule_type {
            "domain_full" => domain == value,
            "domain_suffix" => {
                let suffix = value.trim_start_matches("*.");
                domain == suffix || domain.ends_with(&format!(".{suffix}"))
            }
            "domain_keyword" => domain.contains(value),
            "domain_regex" => {
                Regex::new(value).map(|re| re.is_match(domain)).unwrap_or(false)
            }
            "ip_cidr" => false,
            "process_name" => false,
            "geoip" => {
                if let Some(db) = geo_db {
                    db.domain_in_country(domain, value).await
                } else {
                    false
                }
            }
            "geosite" => {
                if let Some(db) = geo_db {
                    db.domain_in_geosite(domain, value).await
                } else {
                    false
                }
            }
            _ => false,
        };

        if matched {
            matched_rules.push((rule, priority));
        }
    }

    matched_rules.sort_by(|a, b| b.1.cmp(&a.1));

    if let Some((matched_rule, _)) = matched_rules.first() {
        Ok(json!({
            "matched": true,
            "rule_id": matched_rule.get("id"),
            "rule_type": matched_rule.get("rule_type"),
            "rule_value": matched_rule.get("value"),
            "node_id": matched_rule.get("node_id"),
            "priority": matched_rule.get("priority"),
        }))
    } else {
        // Check built-in defaults: Russian traffic → direct, everything else → VPN
        let is_russian = if let Some(db) = geo_db {
            db.domain_in_country(domain, "ru").await ||
            db.domain_in_geosite(domain, "ru").await
        } else {
            false
        };

        if is_russian {
            Ok(json!({
                "matched": true,
                "rule_id": null,
                "rule_type": "geosite",
                "rule_value": "ru",
                "node_id": "direct",
                "priority": -1,
            }))
        } else {
            Ok(json!({
                "matched": false,
                "node_id": "proxy",
                "reason": "No matching rules found, routed through VPN by default"
            }))
        }
    }
}
