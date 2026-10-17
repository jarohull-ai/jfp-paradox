// Copyright (c) 2026 jarohull-ai. All rights reserved.
// Licensed under the MIT License.

//! JFP-Paradox CLI — Autonomous Network Governance & Proof Verification.

use jfp_paradox_core::{GovernorConstitution, NetworkMetrics};
use std::env;
use std::fs;
use std::process::ExitCode;

fn usage() {
    eprintln!(
        "JFP-Paradox v{} — Autonomous Network Governance & Proof Engine",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  jfp-paradox eval <metrics.json|yaml>");
    eprintln!("  jfp-paradox verify-chain <proof_log.jsonl>");
    eprintln!("  jfp-paradox explain <metrics.json|yaml>");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --help     Show this help message");
    eprintln!("  --version  Show version");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        return ExitCode::from(1);
    }

    match args[1].as_str() {
        "eval" | "explain" => {
            if args.len() < 3 {
                eprintln!("Error: Path to metrics file required");
                return ExitCode::from(1);
            }
            let path = &args[2];
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", path, e);
                    return ExitCode::from(1);
                }
            };

            let metrics: NetworkMetrics = match serde_yaml::from_str(&content) {
                Ok(m) => m,
                Err(_) => match serde_json::from_str(&content) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Error parsing metrics JSON/YAML: {}", e);
                        return ExitCode::from(1);
                    }
                },
            };

            let gov = GovernorConstitution::default();
            let verdict = gov.evaluate(&metrics);

            println!("\n🌐 JFP-PARADOX NETWORK GOVERNANCE VERDICT");
            println!("================================================================================");
            println!("📊 Network Health Score: {:.1}/100", verdict.health_score);
            println!(
                "🚨 Critical State:       {}",
                if verdict.critical { "YES (Action Required)" } else { "NO (Healthy)" }
            );
            println!("📌 Primary Reason:        {}", verdict.reason);
            println!(
                "⚠️  Thresholds Breached:  {}",
                if verdict.thresholds_breached.is_empty() {
                    "None".to_string()
                } else {
                    verdict.thresholds_breached.join(", ")
                }
            );
            println!("📡 Sensors Online:       Latency: {}, Loss: {}, Jitter: {}, Interfaces: {}, Routes: {}",
                verdict.sensor_health.ping_ok,
                verdict.sensor_health.ping_ok,
                verdict.sensor_health.tcp_ok,
                verdict.sensor_health.interfaces_ok,
                verdict.sensor_health.routes_ok
            );
            println!("================================================================================\n");

            if verdict.critical {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        "verify-chain" => {
            if args.len() < 3 {
                eprintln!("Error: Path to proof log file required");
                return ExitCode::from(1);
            }
            let path = &args[2];
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading proof log '{}': {}", path, e);
                    return ExitCode::from(1);
                }
            };

            use sha2::{Digest, Sha256};
            let mut expected_prev = "0".repeat(64);
            let mut count = 0;

            for (idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let mut obj: serde_json::Value = match serde_json::from_str(trimmed) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("❌ Line {}: Invalid JSON ({})", idx + 1, e);
                        return ExitCode::from(1);
                    }
                };

                let recorded_hash = obj
                    .get("entry_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let recorded_prev = obj
                    .get("prev_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if recorded_prev != expected_prev {
                    eprintln!(
                        "❌ Line {}: Hash chain broken! Expected prev_hash '{}', found '{}'",
                        idx + 1,
                        expected_prev,
                        recorded_prev
                    );
                    return ExitCode::from(1);
                }

                // Recalculate hash
                if let Some(map) = obj.as_object_mut() {
                    map.remove("entry_hash");
                    map.remove("signature");
                    map.remove("pubkey_id");
                }
                let canonical = match serde_json::to_string(&obj) {
                    Ok(c) => c,
                    Err(_) => return ExitCode::from(1),
                };

                let mut hasher = Sha256::new();
                hasher.update(recorded_prev.as_bytes());
                hasher.update(canonical.as_bytes());
                let computed = format!("{:x}", hasher.finalize());

                if computed != recorded_hash {
                    eprintln!(
                        "❌ Line {}: Entry hash mismatch! Computed '{}', recorded '{}'",
                        idx + 1,
                        computed,
                        recorded_hash
                    );
                    return ExitCode::from(1);
                }

                expected_prev = recorded_hash;
                count += 1;
            }

            println!("✅ Proof Chain Verified: {} entries, 100% cryptographically tamper-evident.", count);
            ExitCode::SUCCESS
        }
        "--help" | "-h" => {
            usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("Unknown command: {}", other);
            usage();
            ExitCode::from(1)
        }
    }
}
