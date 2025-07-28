#!/usr/bin/env python3
"""
Test for cpu_throttle_detector.pxl
This test MUST fail first according to TDD
"""

import os
import sys


def test_cpu_throttle_detector_exists():
    """Test that cpu_throttle_detector.pxl exists"""
    script_path = os.path.join(os.path.dirname(__file__), "..", "src", "cpu_throttle_detector.pxl")
    
    if not os.path.exists(script_path):
        print("❌ EXPECTED FAIL: cpu_throttle_detector.pxl does not exist yet")
        return False
    
    print("✅ PASS: cpu_throttle_detector.pxl exists")
    return True


def test_cpu_throttle_detector_has_required_functions():
    """Test that the script has required functions"""
    script_path = os.path.join(os.path.dirname(__file__), "..", "src", "cpu_throttle_detector.pxl")
    
    if not os.path.exists(script_path):
        print("❌ EXPECTED FAIL: Script doesn't exist yet")
        return False
    
    with open(script_path, 'r') as f:
        content = f.read()
    
    # Check for required elements
    required_elements = [
        "def cpu_throttle_detector():",
        "WEBHOOK_URL",
        "THROTTLE_THRESHOLD",
        "df.cpu_throttled_pct",
        "px.export"
    ]
    
    missing = []
    for element in required_elements:
        if element not in content:
            missing.append(element)
    
    if missing:
        print(f"❌ FAIL: Missing required elements: {missing}")
        return False
    
    print("✅ PASS: All required elements present")
    return True


def main():
    """Run all tests"""
    print("🧪 Testing cpu_throttle_detector.pxl")
    print("="*50)
    
    tests_passed = True
    
    # Test 1: File exists
    tests_passed &= test_cpu_throttle_detector_exists()
    
    # Test 2: Has required functions
    if tests_passed:  # Only run if file exists
        tests_passed &= test_cpu_throttle_detector_has_required_functions()
    
    if tests_passed:
        print("\n✅ All tests passed!")
        sys.exit(0)
    else:
        print("\n❌ Tests failed (as expected in TDD)")
        sys.exit(1)


if __name__ == "__main__":
    main()