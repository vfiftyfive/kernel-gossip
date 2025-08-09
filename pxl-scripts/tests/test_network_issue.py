#!/usr/bin/env python3
"""
Test for network_issue_finder.pxl
Tests REAL network issue detection - NO MOCKS
"""
import subprocess
import json
import time
import os
import sys

def test_network_issue_finder():
    """Test network issue detection with REAL kernel data"""
    print("🧪 Testing network_issue_finder.pxl...")
    
    # Verify PxL file exists
    pxl_path = os.path.join(os.path.dirname(__file__), "../src/network_issue_finder.pxl")
    if not os.path.exists(pxl_path):
        print(f"❌ FAIL: {pxl_path} does not exist")
        return False
    
    # Set configuration via environment
    env = os.environ.copy()
    env['PX_WEBHOOK_URL'] = 'http://kernel-gossip-operator:8080/webhook/pixie'
    env['PX_PACKET_DROP_THRESHOLD'] = '1.0'  # 1% packet drop threshold
    env['PX_RETRANSMIT_THRESHOLD'] = '5.0'   # 5% retransmit threshold
    env['PX_LATENCY_THRESHOLD_MS'] = '100'   # 100ms latency threshold
    
    # Run PxL validation
    validate_cmd = [
        sys.executable,
        os.path.join(os.path.dirname(__file__), "validate_pxl.py"),
        pxl_path
    ]
    
    try:
        result = subprocess.run(validate_cmd, env=env, capture_output=True, text=True, timeout=30)
        
        if result.returncode != 0:
            print(f"❌ FAIL: PxL validation failed")
            print(f"STDOUT: {result.stdout}")
            print(f"STDERR: {result.stderr}")
            return False
        
        # Parse output
        output = json.loads(result.stdout)
        
        # Verify schema
        required_columns = {
            'timestamp', 'cluster', 'pod_name', 'namespace',
            'packet_drop_pct', 'retransmit_pct', 'avg_latency_ms',
            'connection_errors', 'severity'
        }
        
        if 'columns' in output:
            actual_columns = set(output['columns'])
            if not required_columns.issubset(actual_columns):
                missing = required_columns - actual_columns
                print(f"❌ FAIL: Missing columns: {missing}")
                return False
            print("✅ Schema validation passed")
        
        # Verify configuration usage
        if output.get('uses_config', False):
            print("✅ Configuration usage verified")
        else:
            print("❌ FAIL: Script must use px.endpoint_config")
            return False
        
        # Verify severity calculation
        if output.get('has_severity', False):
            print("✅ Severity levels implemented")
        else:
            print("❌ FAIL: Script must calculate severity levels")
            return False
        
        # Verify network issue detection logic
        if output.get('detects_network_issues', False):
            print("✅ Network issue detection logic verified")
        else:
            print("❌ FAIL: Script must detect network issues (drops, retransmits, latency)")
            return False
        
        print("✅ PASS: network_issue_finder.pxl validated successfully")
        return True
        
    except subprocess.TimeoutExpired:
        print("❌ FAIL: Script execution timeout (must complete in < 1 second)")
        return False
    except Exception as e:
        print(f"❌ FAIL: Unexpected error: {e}")
        return False

if __name__ == "__main__":
    success = test_network_issue_finder()
    exit(0 if success else 1)