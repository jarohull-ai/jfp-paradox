#!/usr/bin/env python3
"""Standard test runner for JFP-Paradox without external pytest dependencies."""

import sys
import unittest

# Ensure project root directory is on path
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from daemon.governor import GovernorConstitution
from daemon.supervisor import Supervisor
from daemon.actions import ActionExecutor
from daemon.logger import ProofLogger
from daemon.metrics import snapshot

class TestGovernor(unittest.TestCase):
    def setUp(self):
        self.governor = GovernorConstitution()

    def test_normal_network_not_critical(self):
        metrics = {
            "latency_ms": 20.0,
            "packet_loss_pct": 0.0,
            "jitter_ms": 2.0,
            "bandwidth_utilization_pct": 30.0,
            "sensor_health": {
                "ping_ok": True,
                "tcp_ok": True,
                "interfaces_ok": True,
                "routes_ok": True,
            }
        }
        result = self.governor.evaluate(metrics)
        self.assertFalse(result["critical"])
        self.assertEqual(result["health_score"], 100.0)

    def test_high_latency_triggers_critical(self):
        metrics = {
            "latency_ms": 550.0,
            "packet_loss_pct": 0.0,
            "jitter_ms": 2.0,
            "bandwidth_utilization_pct": 30.0,
            "sensor_health": {
                "ping_ok": True,
                "tcp_ok": True,
                "interfaces_ok": True,
                "routes_ok": True,
            }
        }
        result = self.governor.evaluate(metrics)
        self.assertTrue(result["critical"])
        self.assertLess(result["health_score"], 50.0)

    def test_high_packet_loss_triggers_critical(self):
        metrics = {
            "latency_ms": 20.0,
            "packet_loss_pct": 2.0,
            "jitter_ms": 2.0,
            "bandwidth_utilization_pct": 30.0,
            "sensor_health": {
                "ping_ok": True,
                "tcp_ok": True,
                "interfaces_ok": True,
                "routes_ok": True,
            }
        }
        result = self.governor.evaluate(metrics)
        self.assertIn("packet_loss_high", result["thresholds_breached"])

    def test_missing_sensor_is_critical(self):
        metrics = {
            "latency_ms": 20.0,
            "packet_loss_pct": 0.0,
            "jitter_ms": 2.0,
            "bandwidth_utilization_pct": 30.0,
            "sensor_health": {
                "ping_ok": False,
                "tcp_ok": True,
                "interfaces_ok": True,
                "routes_ok": True,
            }
        }
        result = self.governor.evaluate(metrics)
        self.assertTrue(result["critical"])
        self.assertEqual(result["health_score"], 0.0)

    def test_action_governance_rules(self):
        # Healthy network -> tuning rejected
        normal = {
            "latency_ms": 20.0,
            "packet_loss_pct": 0.0,
            "jitter_ms": 2.0,
            "bandwidth_utilization_pct": 30.0,
            "sensor_health": {"ping_ok": True, "tcp_ok": True, "interfaces_ok": True, "routes_ok": True}
        }
        allowed, reason = self.governor.approve_action("set_qdisc", normal, {})
        self.assertFalse(allowed)
        self.assertEqual(reason, "deny_tuning_network_healthy")

        # Emergency stop always allowed
        allowed, reason = self.governor.approve_action("panic_stop_network", normal, {})
        self.assertTrue(allowed)

        # Degraded network -> tuning allowed
        degraded = {
            "latency_ms": 350.0,
            "packet_loss_pct": 2.0,
            "jitter_ms": 40.0,
            "bandwidth_utilization_pct": 80.0,
            "sensor_health": {"ping_ok": True, "tcp_ok": True, "interfaces_ok": True, "routes_ok": True}
        }
        allowed, reason = self.governor.approve_action("set_qdisc", degraded, {})
        self.assertTrue(allowed)


class TestSupervisor(unittest.TestCase):
    def setUp(self):
        self.supervisor = Supervisor()

    def test_recommendation_under_critical_metrics(self):
        degraded = {
            "ts": "2026-10-17T12:00:00Z",
            "latency_ms": 350.0,
            "packet_loss_pct": 3.5,
            "jitter_ms": 90.0,
            "bandwidth_utilization_pct": 85.0
        }
        rec = self.supervisor.recommend(degraded)
        self.assertIsNotNone(rec)
        self.assertEqual(rec["level"], "L3")
        self.assertEqual(rec["name"], "set_qdisc")

    def test_recommendation_under_normal_metrics(self):
        normal = {
            "ts": "2026-10-17T12:00:00Z",
            "latency_ms": 20.0,
            "packet_loss_pct": 0.0,
            "jitter_ms": 2.0,
            "bandwidth_utilization_pct": 20.0
        }
        rec = self.supervisor.recommend(normal)
        self.assertIsNone(rec)


class TestProofChain(unittest.TestCase):
    def test_tamper_evident_chain(self):
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".jsonl", delete=False) as f:
            log_path = f.name

        logger = ProofLogger(log_path, "0" * 64)
        e1 = logger.append({"event": "genesis"})
        e2 = logger.append({"event": "step1"})

        self.assertEqual(e1["counter"], 0)
        self.assertEqual(e2["counter"], 1)
        self.assertEqual(e2["prev_hash"], e1["entry_hash"])


class TestActionExecutor(unittest.TestCase):
    def setUp(self):
        self.executor = ActionExecutor(dry_run=True)

    def test_dry_run_execution(self):
        ok, msg = self.executor.execute("set_qdisc", {"interface": "eth0", "qdisc_type": "fq_codel"})
        self.assertTrue(ok)
        self.assertIn("[DRY_RUN]", msg)

    def test_disallowed_action(self):
        ok, msg = self.executor.execute("rm_rf_root", {})
        self.assertFalse(ok)
        self.assertIn("not in allowlist", msg)


if __name__ == "__main__":
    unittest.main()
