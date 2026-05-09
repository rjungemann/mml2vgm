#!/usr/bin/env python3
"""
Golden Master Metadata Manager

Manages the golden master database (metadata.json) for tracking validation progress.
Provides functions to:
  - Load and save metadata
  - Update test results
  - Generate validation reports
  - Calculate pass rates
"""

import json
import argparse
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, Optional


class MetadataManager:
    """Manage golden master validation metadata."""

    def __init__(self, metadata_path: str):
        self.metadata_path = Path(metadata_path)
        self.data = self._load()

    def _load(self) -> Dict[str, Any]:
        """Load metadata from JSON file."""
        if not self.metadata_path.exists():
            raise FileNotFoundError(f"Metadata file not found: {self.metadata_path}")

        with open(self.metadata_path, 'r') as f:
            return json.load(f)

    def save(self) -> None:
        """Save metadata to JSON file."""
        with open(self.metadata_path, 'w') as f:
            json.dump(self.data, f, indent=2)
        print(f"✓ Metadata saved to {self.metadata_path}")

    def update_test_result(
        self,
        chip: str,
        test_name: str,
        status: str,
        metrics: Dict[str, Any],
        notes: str = ""
    ) -> None:
        """
        Update a test result in metadata.

        Args:
            chip: Chip name (e.g., 'ym2151')
            test_name: Test name (e.g., 'envelope')
            status: Status ('passed', 'failed', 'pending')
            metrics: Dictionary of metric results
            notes: Optional notes
        """
        if chip not in self.data or chip == "summary":
            raise ValueError(f"Unknown chip: {chip}")

        if test_name not in self.data[chip]["tests"]:
            raise ValueError(f"Unknown test: {test_name} for chip {chip}")

        test = self.data[chip]["tests"][test_name]
        test["validation_status"] = status
        test["metrics"].update(metrics)
        test["passed"] = (status == "passed")
        test["generated_date"] = datetime.now().isoformat()
        if notes:
            test["notes"] = notes

        self._update_chip_summary(chip)
        self._update_global_summary()

    def _update_chip_summary(self, chip: str) -> None:
        """Recalculate summary for a chip."""
        tests = self.data[chip]["tests"].values()
        total = len(tests)
        passed = sum(1 for t in tests if t.get("passed") is True)
        failed = sum(1 for t in tests if t.get("passed") is False)
        pending = sum(1 for t in tests if t.get("passed") is None)

        self.data[chip]["summary"] = {
            "total_tests": total,
            "passed": passed,
            "failed": failed,
            "pending": pending,
            "pass_rate_percent": int((passed / total * 100)) if total > 0 else None,
            "overall_status": "passed" if passed == total else (
                "in_progress" if pending > 0 else "failed"
            )
        }

    def _update_global_summary(self) -> None:
        """Recalculate global summary."""
        total_tests = 0
        total_passed = 0
        total_failed = 0
        total_pending = 0

        for chip_name in ["ym2151", "ym2203", "ym2608", "opl", "segapcm", "nes", "qsound"]:
            if chip_name in self.data:
                summary = self.data[chip_name]["summary"]
                total_tests += summary["total_tests"]
                total_passed += summary["passed"]
                total_failed += summary["failed"]
                total_pending += summary["pending"]

        phase_2_complete = total_pending == 0 and total_tests == 7  # YM2151 + YM2203

        self.data["summary"] = {
            "total_chips": 7,
            "total_tests": total_tests,
            "tests_passed": total_passed,
            "tests_pending": total_pending,
            "tests_failed": total_failed,
            "overall_pass_rate_percent": int((total_passed / total_tests * 100)) if total_tests > 0 else 0,
            "phase_1_status": "infrastructure_complete",
            "phase_2_status": "complete" if phase_2_complete else "in_progress"
        }

    def get_chip_status(self, chip: str) -> Dict[str, Any]:
        """Get full status for a chip."""
        if chip not in self.data:
            raise ValueError(f"Unknown chip: {chip}")

        return {
            "chip": chip,
            "summary": self.data[chip]["summary"],
            "tests": self.data[chip]["tests"]
        }

    def get_test_result(self, chip: str, test_name: str) -> Dict[str, Any]:
        """Get result for a specific test."""
        if chip not in self.data:
            raise ValueError(f"Unknown chip: {chip}")

        if test_name not in self.data[chip]["tests"]:
            raise ValueError(f"Unknown test: {test_name}")

        return self.data[chip]["tests"][test_name]

    def print_report(self, chip: Optional[str] = None) -> None:
        """Print a formatted validation report."""
        if chip:
            self._print_chip_report(chip)
        else:
            self._print_global_report()

    def _print_global_report(self) -> None:
        """Print global validation report."""
        summary = self.data["summary"]

        print("\n" + "="*60)
        print("GOLDEN MASTER VALIDATION REPORT")
        print("="*60)
        print(f"\nTotal Chips: {summary['total_chips']}")
        print(f"Total Tests: {summary['total_tests']}")
        print(f"Passed: {summary['tests_passed']}")
        print(f"Failed: {summary['tests_failed']}")
        print(f"Pending: {summary['tests_pending']}")
        print(f"Overall Pass Rate: {summary['overall_pass_rate_percent']}%")
        print(f"Phase 1 Status: {summary['phase_1_status']}")
        print(f"Phase 2 Status: {summary['phase_2_status']}")

        print("\n" + "-"*60)
        print("CHIP SUMMARY")
        print("-"*60)

        for chip_name in ["ym2151", "ym2203", "ym2608", "opl", "segapcm", "nes", "qsound"]:
            if chip_name in self.data and chip_name != "summary":
                chip_data = self.data[chip_name]
                summary = chip_data["summary"]
                status_symbol = "✓" if summary["overall_status"] == "passed" else (
                    "⏳" if summary["overall_status"] == "in_progress" else "✗"
                )

                print(f"\n{status_symbol} {chip_name.upper()}")
                print(f"  Tests: {summary['passed']}/{summary['total_tests']} passed")
                print(f"  Pass Rate: {summary['pass_rate_percent']}%")
                print(f"  Status: {summary['overall_status']}")

    def _print_chip_report(self, chip: str) -> None:
        """Print validation report for a specific chip."""
        if chip not in self.data:
            print(f"Unknown chip: {chip}")
            return

        chip_data = self.data[chip]
        summary = chip_data["summary"]

        print("\n" + "="*60)
        print(f"{chip.upper()} VALIDATION REPORT")
        print("="*60)
        print(f"\nChip: {chip_data.get('chip_name', chip)}")
        print(f"Reference Emulator: {chip_data.get('reference_emulator', 'N/A')}")
        print(f"Validation Method: {chip_data.get('validation_method', 'N/A')}")

        print(f"\nSummary:")
        print(f"  Total Tests: {summary['total_tests']}")
        print(f"  Passed: {summary['passed']}")
        print(f"  Failed: {summary['failed']}")
        print(f"  Pending: {summary['pending']}")
        print(f"  Pass Rate: {summary['pass_rate_percent']}%")
        print(f"  Status: {summary['overall_status']}")

        print(f"\n" + "-"*60)
        print("TEST RESULTS")
        print("-"*60)

        for test_name, test_data in chip_data["tests"].items():
            status_symbol = "✓" if test_data["passed"] else (
                "⏳" if test_data["validation_status"] == "pending" else "✗"
            )

            print(f"\n{status_symbol} {test_name}")
            print(f"  Description: {test_data.get('description', 'N/A')}")
            print(f"  Status: {test_data['validation_status']}")

            if test_data.get("metrics"):
                print(f"  Metrics:")
                for metric_name, metric_value in test_data["metrics"].items():
                    if metric_value is not None:
                        print(f"    - {metric_name}: {metric_value}")

            if test_data.get("notes"):
                print(f"  Notes: {test_data['notes']}")


def main():
    parser = argparse.ArgumentParser(
        description="Manage golden master validation metadata"
    )
    parser.add_argument(
        "action",
        choices=["report", "update", "status"],
        help="Action to perform"
    )
    parser.add_argument(
        "--chip",
        help="Chip name (ym2151, ym2203, etc.)"
    )
    parser.add_argument(
        "--test",
        help="Test name (envelope, fm, etc.)"
    )
    parser.add_argument(
        "--status",
        choices=["passed", "failed", "pending"],
        help="Test status"
    )
    parser.add_argument(
        "--metrics",
        help="JSON string of metrics (e.g., '{\"correlation\": 0.96}')"
    )
    parser.add_argument(
        "--notes",
        help="Notes about the test result"
    )
    parser.add_argument(
        "--metadata",
        default="tests/golden_master/metadata.json",
        help="Path to metadata.json"
    )

    args = parser.parse_args()

    try:
        manager = MetadataManager(args.metadata)

        if args.action == "report":
            manager.print_report(args.chip)

        elif args.action == "status":
            if args.chip:
                status = manager.get_chip_status(args.chip)
                print(json.dumps(status, indent=2))
            else:
                status = manager.data.get("summary", {})
                print(json.dumps(status, indent=2))

        elif args.action == "update":
            if not all([args.chip, args.test, args.status]):
                print("Error: --chip, --test, and --status are required for update")
                return 1

            metrics = {}
            if args.metrics:
                metrics = json.loads(args.metrics)

            manager.update_test_result(
                args.chip,
                args.test,
                args.status,
                metrics,
                args.notes or ""
            )
            manager.save()
            manager.print_report(args.chip)

        return 0

    except Exception as e:
        print(f"Error: {e}")
        return 1


if __name__ == "__main__":
    exit(main())
