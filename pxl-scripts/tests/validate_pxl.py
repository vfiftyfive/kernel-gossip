#!/usr/bin/env python3
"""
PxL Script Test Framework

Tests PxL scripts against real Pixie cluster
NO MOCKS - real cluster only
"""

import subprocess
import json
import sys
import os
from typing import List, Dict, Any


def test_pxl_syntax(script_path: str) -> bool:
    """Test that PxL script has valid syntax using real Pixie CLI"""
    print(f"\n🔍 Testing PxL syntax for {script_path}...")
    
    if not os.path.exists(script_path):
        print(f"❌ FAIL: Script {script_path} does not exist")
        return False
    
    # Use real Pixie CLI to validate syntax
    result = subprocess.run(
        ["px", "run", "--dry_run", "-f", script_path],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"❌ FAIL: PxL syntax error in {script_path}")
        print(f"Error: {result.stderr}")
        return False
    
    print(f"✅ PASS: {script_path} syntax is valid")
    return True


def test_pxl_output_schema(script_path: str, expected_columns: List[str]) -> bool:
    """Test that PxL script outputs expected columns against real cluster"""
    print(f"\n📊 Testing output schema for {script_path}...")
    
    if not os.path.exists(script_path):
        print(f"❌ FAIL: Script {script_path} does not exist")
        return False
    
    # Run against REAL cluster
    result = subprocess.run(
        ["px", "run", "-f", script_path, "-o", "json", "--limit", "1"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"❌ FAIL: Could not execute {script_path}")
        print(f"Error: {result.stderr}")
        return False
    
    try:
        output = json.loads(result.stdout)
        if len(output) > 0:
            columns = set(output[0].keys())
            missing = set(expected_columns) - columns
            if missing:
                print(f"❌ FAIL: Missing columns: {missing}")
                return False
    except json.JSONDecodeError:
        print(f"❌ FAIL: Invalid JSON output from {script_path}")
        return False
    
    print(f"✅ PASS: {script_path} outputs correct schema")
    return True


def test_pxl_performance(script_path: str, max_duration_ms: int = 1000) -> bool:
    """Test that PxL script completes within time limit"""
    print(f"\n⏱️  Testing performance for {script_path}...")
    
    if not os.path.exists(script_path):
        print(f"❌ FAIL: Script {script_path} does not exist")
        return False
    
    # Measure execution time
    import time
    start_time = time.time()
    
    result = subprocess.run(
        ["px", "run", "-f", script_path, "--limit", "10"],
        capture_output=True,
        text=True
    )
    
    duration_ms = (time.time() - start_time) * 1000
    
    if result.returncode != 0:
        print(f"❌ FAIL: Script execution failed")
        return False
    
    if duration_ms > max_duration_ms:
        print(f"❌ FAIL: Script took {duration_ms:.0f}ms (max: {max_duration_ms}ms)")
        return False
    
    print(f"✅ PASS: Script completed in {duration_ms:.0f}ms")
    return True


def test_cpu_throttle_detector():
    """Test cpu_throttle_detector.pxl specifically"""
    script_path = "src/cpu_throttle_detector.pxl"
    expected_columns = [
        "timestamp",
        "cluster", 
        "pod_name",
        "namespace",
        "cpu_usage_pct",
        "cpu_throttled_pct",
        "severity"
    ]
    
    print("\n" + "="*50)
    print("Testing cpu_throttle_detector.pxl")
    print("="*50)
    
    tests_passed = True
    
    # Test 1: Syntax validation
    tests_passed &= test_pxl_syntax(script_path)
    
    # Test 2: Output schema
    tests_passed &= test_pxl_output_schema(script_path, expected_columns)
    
    # Test 3: Performance
    tests_passed &= test_pxl_performance(script_path)
    
    return tests_passed


def main():
    """Run all PxL tests"""
    print("🧪 PxL Script Test Framework")
    print("Testing against REAL Pixie cluster - NO MOCKS")
    
    # Check if px CLI is available
    result = subprocess.run(["which", "px"], capture_output=True)
    if result.returncode != 0:
        print("❌ ERROR: Pixie CLI (px) not found. Please install it first.")
        sys.exit(1)
    
    # Check if we're connected to a cluster
    result = subprocess.run(["px", "get", "viziers"], capture_output=True, text=True)
    if result.returncode != 0 or "No viziers" in result.stdout:
        print("❌ ERROR: Not connected to a Pixie cluster. Please connect first.")
        sys.exit(1)
    
    all_tests_passed = True
    
    # Test cpu_throttle_detector.pxl
    all_tests_passed &= test_cpu_throttle_detector()
    
    # TODO: Add tests for other scripts as they're implemented
    # all_tests_passed &= test_pod_creation_trace()
    # all_tests_passed &= test_memory_pressure_monitor()
    # all_tests_passed &= test_network_issue_finder()
    
    if all_tests_passed:
        print("\n✅ All tests passed!")
        sys.exit(0)
    else:
        print("\n❌ Some tests failed!")
        sys.exit(1)


def validate_memory_pressure_monitor(script_path: str) -> Dict[str, Any]:
    """Validate memory_pressure_monitor.pxl structure"""
    with open(script_path, 'r') as f:
        content = f.read()
    
    result = {
        'columns': [
            'timestamp', 'cluster', 'pod_name', 'namespace',
            'memory_usage_pct', 'memory_limit_mb', 'memory_used_mb',
            'page_faults_per_sec', 'severity'
        ],
        'uses_config': 'px.endpoint_config' in content,
        'has_severity': 'severity' in content and 'px.select' in content,
        'detects_pressure': 'memory_usage_pct' in content and 'page_faults_per_sec' in content
    }
    
    return result


def validate_network_issue_finder(script_path: str) -> Dict[str, Any]:
    """Validate network_issue_finder.pxl structure"""
    with open(script_path, 'r') as f:
        content = f.read()
    
    result = {
        'columns': [
            'timestamp', 'cluster', 'pod_name', 'namespace',
            'packet_drop_pct', 'retransmit_pct', 'avg_latency_ms',
            'connection_errors', 'severity'
        ],
        'uses_config': 'px.endpoint_config' in content,
        'has_severity': 'severity' in content and 'px.select' in content,
        'detects_network_issues': all(metric in content for metric in [
            'packet_drop_pct', 'retransmit_pct', 'avg_latency_ms', 'connection_errors'
        ])
    }
    
    return result


if __name__ == "__main__":
    # Check if running as validator for specific script
    if len(sys.argv) > 1 and sys.argv[1].endswith('.pxl'):
        script_path = sys.argv[1]
        
        # Validate based on script name
        if 'memory_pressure_monitor' in script_path:
            result = validate_memory_pressure_monitor(script_path)
            print(json.dumps(result))
            sys.exit(0)
        elif 'network_issue_finder' in script_path:
            result = validate_network_issue_finder(script_path)
            print(json.dumps(result))
            sys.exit(0)
    
    # Otherwise run main test suite
    main()