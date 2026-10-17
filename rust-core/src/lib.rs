// Copyright (c) 2026 jarohull-ai. All rights reserved.
// Licensed under the MIT License.

//! Core types, Governor, Supervisor, and ProofLogger for JFP-Paradox.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Health status of individual network sensors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SensorHealth {
    pub ping_ok: bool,
    pub tcp_ok: bool,
    pub interfaces_ok: bool,
    pub routes_ok: bool,
    #[serde(default)]
    pub latency_ok: bool,
    #[serde(default)]
    pub loss_ok: bool,
    #[serde(default)]
    pub jitter_ok: bool,
    #[serde(default)]
    pub bandwidth_ok: bool,
}

impl Default for SensorHealth {
    fn default() -> Self {
        Self {
            ping_ok: true,
            tcp_ok: true,
            interfaces_ok: true,
            routes_ok: true,
            latency_ok: true,
            loss_ok: true,
            jitter_ok: true,
            bandwidth_ok: true,
        }
    }
}

/// Instantaneous network metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    #[serde(default = "default_timestamp")]
    pub ts: String,
    pub latency_ms: f64,
    pub packet_loss_pct: f64,
    pub jitter_ms: f64,
    pub bandwidth_utilization_pct: f64,
    #[serde(default)]
    pub sensor_health: SensorHealth,
}

fn default_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Evaluation result from the Governor Constitution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorVerdict {
    pub health_score: f64,
    pub critical: bool,
    pub reason: String,
    pub thresholds_breached: Vec<String>,
    pub sensor_health: SensorHealth,
}

/// Hard Constitutional Governor for Network Safety.
pub struct GovernorConstitution {
    pub latency_critical_ms: f64,
    pub latency_high_ms: f64,
    pub packet_loss_critical_pct: f64,
    pub packet_loss_high_pct: f64,
    pub jitter_critical_ms: f64,
    pub jitter_high_ms: f64,
    pub bandwidth_utilization_critical_pct: f64,
}

impl Default for GovernorConstitution {
    fn default() -> Self {
        Self {
            latency_critical_ms: 500.0,
            latency_high_ms: 200.0,
            packet_loss_critical_pct: 5.0,
            packet_loss_high_pct: 1.0,
            jitter_critical_ms: 100.0,
            jitter_high_ms: 30.0,
            bandwidth_utilization_critical_pct: 95.0,
        }
    }
}

impl GovernorConstitution {
    /// Calculate network health gradient: g(s, h_{n+1}).
    pub fn gradient(current: &NetworkMetrics, next_snap: &NetworkMetrics) -> f64 {
        let latency_delta = next_snap.latency_ms - current.latency_ms;
        let loss_delta = next_snap.packet_loss_pct - current.packet_loss_pct;
        let jitter_delta = next_snap.jitter_ms - current.jitter_ms;

        let g = (latency_delta * 0.5) + (loss_delta * 2.0) + (jitter_delta * 0.3);
        (g * 1000.0).round() / 1000.0
    }

    /// Evaluates network health and determines criticality.
    pub fn evaluate(&self, metrics: &NetworkMetrics) -> GovernorVerdict {
        let latency_ok = metrics.sensor_health.ping_ok;
        let loss_ok = metrics.sensor_health.ping_ok;
        let jitter_ok = metrics.sensor_health.tcp_ok;
        let interfaces_ok = metrics.sensor_health.interfaces_ok;
        let routes_ok = metrics.sensor_health.routes_ok;
        let bandwidth_ok =
            metrics.bandwidth_utilization_pct < self.bandwidth_utilization_critical_pct;

        let sensors_healthy =
            latency_ok && loss_ok && jitter_ok && interfaces_ok && routes_ok;

        if !sensors_healthy {
            return GovernorVerdict {
                health_score: 0.0,
                critical: true,
                reason: "sensor_missing".to_string(),
                thresholds_breached: vec!["sensor_health".to_string()],
                sensor_health: SensorHealth {
                    ping_ok: metrics.sensor_health.ping_ok,
                    tcp_ok: metrics.sensor_health.tcp_ok,
                    interfaces_ok,
                    routes_ok,
                    latency_ok,
                    loss_ok,
                    jitter_ok,
                    bandwidth_ok,
                },
            };
        }

        let mut breached = Vec::new();
        let mut health_score: f64 = 100.0;

        if metrics.latency_ms >= self.latency_critical_ms {
            breached.push("latency_critical".to_string());
            health_score -= 55.0;
        } else if metrics.latency_ms >= self.latency_high_ms {
            breached.push("latency_high".to_string());
            health_score -= 20.0;
        }

        if metrics.packet_loss_pct >= self.packet_loss_critical_pct {
            breached.push("packet_loss_critical".to_string());
            health_score -= 50.0;
        } else if metrics.packet_loss_pct >= self.packet_loss_high_pct {
            breached.push("packet_loss_high".to_string());
            health_score -= 25.0;
        }

        if metrics.jitter_ms >= self.jitter_critical_ms {
            breached.push("jitter_critical".to_string());
            health_score -= 30.0;
        } else if metrics.jitter_ms >= self.jitter_high_ms {
            breached.push("jitter_high".to_string());
            health_score -= 15.0;
        }

        if metrics.bandwidth_utilization_pct >= self.bandwidth_utilization_critical_pct {
            breached.push("bandwidth_critical".to_string());
            health_score -= 20.0;
        }

        let health_clamped = health_score.clamp(0.0, 100.0);
        let has_critical_breach = breached.iter().any(|b| b.ends_with("_critical"));
        let is_critical = health_clamped < 50.0 || breached.len() >= 2 || has_critical_breach;

        let mut reason = "healthy".to_string();
        if let Some(first_breach) = breached.first() {
            reason = first_breach.clone();
        }
        if is_critical && reason.contains("critical") {
            reason = "critical_state".to_string();
        }

        GovernorVerdict {
            health_score: (health_clamped * 100.0).round() / 100.0,
            critical: is_critical,
            reason,
            thresholds_breached: breached,
            sensor_health: SensorHealth {
                ping_ok: metrics.sensor_health.ping_ok,
                tcp_ok: metrics.sensor_health.tcp_ok,
                interfaces_ok,
                routes_ok,
                latency_ok,
                loss_ok,
                jitter_ok,
                bandwidth_ok,
            },
        }
    }

    /// Evaluates if an action is constitutionally permitted.
    pub fn approve_action(&self, action_name: &str, metrics: &NetworkMetrics) -> (bool, String) {
        let verdict = self.evaluate(metrics);

        // Emergency stops always allowed
        if matches!(
            action_name,
            "panic_stop_network" | "reset_all_rules" | "failsafe_revert"
        ) {
            return (true, "allow_emergency_action".to_string());
        }

        // Monitoring always allowed
        if matches!(action_name, "get_network_state" | "analyze_traffic") {
            return (true, "allow_monitoring".to_string());
        }

        // Tuning actions only if degraded
        if matches!(
            action_name,
            "set_qdisc" | "set_tcp_window" | "adjust_iptables" | "priority_queue"
        ) {
            if verdict.health_score > 70.0 {
                return (false, "deny_tuning_network_healthy".to_string());
            }
            return (true, "allow_tuning_degraded".to_string());
        }

        // Route failover requires critical state
        if action_name == "route_failover" {
            if !verdict.critical {
                return (false, "deny_failover_not_critical".to_string());
            }
            return (true, "allow_failover_critical".to_string());
        }

        (false, format!("deny_unknown_action_{}", action_name))
    }
}

/// Action recommendation from Adaptive Supervisor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorRecommendation {
    pub level: String, // "L1", "L2", "L3"
    pub name: String,
    pub params: serde_json::Value,
    pub reason: String,
}

/// Adaptive Network Supervisor.
pub struct Supervisor {
    pub window: VecDeque<NetworkMetrics>,
    pub enabled: bool,
    pub warn_threshold_latency_increasing: f64,
    pub warn_threshold_loss_increasing: f64,
    pub warn_threshold_jitter_increasing: f64,
}

impl Default for Supervisor {
    fn default() -> Self {
        Self {
            window: VecDeque::with_capacity(600),
            enabled: true,
            warn_threshold_latency_increasing: 10.0,
            warn_threshold_loss_increasing: 0.5,
            warn_threshold_jitter_increasing: 5.0,
        }
    }
}

impl Supervisor {
    /// Update sliding window with latest sample.
    pub fn update(&mut self, metrics: NetworkMetrics) {
        if self.window.len() >= 600 {
            self.window.pop_front();
        }
        self.window.push_back(metrics);
    }

    /// Latency trend per second over window.
    pub fn latency_trend(&self) -> f64 {
        if self.window.len() < 2 {
            return 0.0;
        }
        let newest = self.window.back().unwrap().latency_ms;
        let oldest = self.window.front().unwrap().latency_ms;
        let dt = (self.window.len() as f64).max(1.0);
        ((newest - oldest) / dt * 1000.0).round() / 1000.0
    }

    /// Recommends a network tuning action based on state and trends.
    pub fn recommend(&self, metrics: &NetworkMetrics) -> Option<SupervisorRecommendation> {
        if !self.enabled {
            return None;
        }

        // L3: Critical
        if metrics.latency_ms > 300.0
            || metrics.packet_loss_pct > 3.0
            || metrics.jitter_ms > 80.0
            || metrics.bandwidth_utilization_pct > 90.0
        {
            return Some(SupervisorRecommendation {
                level: "L3".to_string(),
                name: "set_qdisc".to_string(),
                params: serde_json::json!({
                    "interface": "eth0",
                    "qdisc_type": "fq_codel"
                }),
                reason: format!(
                    "critical_state latency={}ms loss={}%",
                    metrics.latency_ms, metrics.packet_loss_pct
                ),
            });
        }

        // L2: High degradation
        if metrics.latency_ms > 200.0 || metrics.packet_loss_pct > 1.5 || metrics.jitter_ms > 50.0 {
            return Some(SupervisorRecommendation {
                level: "L2".to_string(),
                name: "set_tcp_window".to_string(),
                params: serde_json::json!({ "size_kb": 256 }),
                reason: format!(
                    "high_degradation latency={}ms jitter={}ms",
                    metrics.latency_ms, metrics.jitter_ms
                ),
            });
        }

        None
    }

    /// Supervisor approval for proposed action.
    pub fn approve_action(&self, action_name: &str, metrics: &NetworkMetrics) -> (bool, String) {
        if matches!(action_name, "panic_stop_network" | "reset_all_rules") {
            return (true, "allow_emergency".to_string());
        }

        if action_name == "route_failover" {
            if metrics.packet_loss_pct < 2.0 && metrics.latency_ms < 250.0 {
                return (false, "deny_failover_metrics_not_bad_enough".to_string());
            }
            return (true, "allow_failover_critical".to_string());
        }

        if action_name == "set_qdisc" {
            if metrics.bandwidth_utilization_pct < 80.0 && metrics.latency_ms < 300.0 {
                return (false, "deny_qdisc_premature".to_string());
            }
            return (true, "allow_qdisc_needed".to_string());
        }

        (true, "allow_supervisor".to_string())
    }
}

/// Cryptographically secure, append-only Proof Logger (SHA-256 hash chain).
pub struct ProofLogger {
    pub log_path: PathBuf,
    pub prev_hash: String,
    pub counter: u64,
}

impl ProofLogger {
    pub fn new(log_path: PathBuf, seed_hash: String) -> Self {
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self {
            log_path,
            prev_hash: seed_hash,
            counter: 0,
        }
    }

    /// Append record to SHA-256 proof chain.
    pub fn append(&mut self, mut record: serde_json::Value) -> Result<serde_json::Value, String> {
        let prev_hash = self.prev_hash.clone();
        let counter = self.counter;

        if let Some(obj) = record.as_object_mut() {
            obj.insert("counter".to_string(), serde_json::json!(counter));
            obj.insert("prev_hash".to_string(), serde_json::json!(prev_hash));
        } else {
            return Err("Record must be a JSON object".to_string());
        }

        let canonical_str = serde_json::to_string(&record).map_err(|e| e.to_string())?;

        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(canonical_str.as_bytes());
        let entry_hash = format!("{:x}", hasher.finalize());

        if let Some(obj) = record.as_object_mut() {
            obj.insert("entry_hash".to_string(), serde_json::json!(entry_hash));
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| e.to_string())?;

        let final_str = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        writeln!(file, "{}", final_str).map_err(|e| e.to_string())?;

        self.prev_hash = entry_hash;
        self.counter += 1;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governor_normal_and_critical() {
        let gov = GovernorConstitution::default();
        let normal = NetworkMetrics {
            ts: default_timestamp(),
            latency_ms: 20.0,
            packet_loss_pct: 0.0,
            jitter_ms: 2.0,
            bandwidth_utilization_pct: 30.0,
            sensor_health: SensorHealth::default(),
        };

        let v1 = gov.evaluate(&normal);
        assert!(!v1.critical);
        assert_eq!(v1.health_score, 100.0);

        let degraded = NetworkMetrics {
            ts: default_timestamp(),
            latency_ms: 550.0,
            packet_loss_pct: 2.0,
            jitter_ms: 40.0,
            bandwidth_utilization_pct: 85.0,
            sensor_health: SensorHealth::default(),
        };

        let v2 = gov.evaluate(&degraded);
        assert!(v2.critical);
        assert!(v2.health_score < 50.0);
    }

    #[test]
    fn test_proof_logger_chain() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_file = temp_dir.path().join("proof.jsonl");

        let mut logger = ProofLogger::new(log_file.clone(), "0".repeat(64));
        let r1 = logger.append(serde_json::json!({ "event": "init" })).unwrap();
        let r2 = logger.append(serde_json::json!({ "event": "tune" })).unwrap();

        assert_eq!(r1["counter"], 0);
        assert_eq!(r2["counter"], 1);
        assert_eq!(r2["prev_hash"], r1["entry_hash"]);
    }
}
