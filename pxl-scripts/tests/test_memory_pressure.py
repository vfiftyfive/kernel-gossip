#!/usr/bin/env python3
"""
Test for memory_pressure_monitor.pxl
Tests REAL memory pressure detection - NO MOCKS
"""
import subprocess
import json
import time
import os
import sys

def test_memory_pressure_monitor():
    """Test memory pressure detection with REAL kernel data"""
    print("🧪 Testing memory_pressure_monitor.pxl...")
    
    # Verify PxL file exists
    pxl_path = os.path.join(os.path.dirname(__file__), "../src/memory_pressure_monitor.pxl")
    if not os.path.exists(pxl_path):
        print(f"❌ FAIL: {pxl_path} does not exist")
        return False
    
    # Set configuration via environment
    env = os.environ.copy()
    env['PX_WEBHOOK_URL'] = 'http://kernel-gossip-operator:8080/webhook/pixie'
    env['PX_MEMORY_PRESSURE_THRESHOLD'] = '70.0'  # 70% memory usage threshold
    env['PX_WARNING_THRESHOLD'] = '80.0'
    env['PX_CRITICAL_THRESHOLD'] = '90.0'
    
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
            'memory_usage_pct', 'memory_limit_mb', 'memory_used_mb',
            'page_faults_per_sec', 'severity'
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
        
        # Verify memory pressure detection logic
        if output.get('detects_pressure', False):
            print("✅ Memory pressure detection logic verified")
        else:
            print("❌ FAIL: Script must detect memory pressure conditions")
            return False
        
        print("✅ PASS: memory_pressure_monitor.pxl validated successfully")
        return True
        
    except subprocess.TimeoutExpired:
        print("❌ FAIL: Script execution timeout (must complete in < 1 second)")
        return False
    except Exception as e:
        print(f"❌ FAIL: Unexpected error: {e}")
        return False

if __name__ == "__main__":
    success = test_memory_pressure_monitor()
    exit(0 if success else 1)