#!/usr/bin/env python3
"""
Final verification script for Rusteze v1.0.0
Runs comprehensive tests to ensure everything works before release.
"""

import sys
import subprocess
import os

def run_test(name, command, cwd=None):
    """Run a test and return success status."""
    print(f"\n{'='*60}")
    print(f"Test: {name}")
    print(f"{'='*60}")
    print(f"Command: {command}")
    print(f"Working directory: {cwd or os.getcwd()}")
    
    try:
        result = subprocess.run(
            command,
            shell=True,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=300
        )
        
        if result.returncode == 0:
            print(f"✅ PASSED: {name}")
            if result.stdout:
                print("\nOutput:")
                print(result.stdout[:500])  # Show first 500 chars
            return True
        else:
            print(f"❌ FAILED: {name}")
            if result.stderr:
                print("\nError:")
                print(result.stderr[:1000])
            return False
    except subprocess.TimeoutExpired:
        print(f"⏱️ TIMEOUT: {name}")
        return False
    except Exception as e:
        print(f"❌ ERROR: {name} - {e}")
        return False

def main():
    """Run all verification tests."""
    print("="*60)
    print("Rusteze v1.0.0 Release Verification")
    print("="*60)
    
    rusteze_dir = os.path.dirname(os.path.abspath(__file__))
    core_dir = os.path.join(rusteze_dir, "rusteze-core")
    
    tests = [
        ("Rust compilation", "cargo check", core_dir),
        ("Rust tests", "cargo test --lib", core_dir),
        ("Python import", "python -c 'import rusteze; print(rusteze.__file__)'", rusteze_dir),
        ("Environment creation", "python -c 'import rusteze; env = rusteze.Env(seed=42); print(\"OK\")'", rusteze_dir),
        ("Environment reset", "python -c 'import rusteze; env = rusteze.Env(seed=42); obs = env.reset(); print(f\"Shape: {obs.shape}\")'", rusteze_dir),
        ("Environment step", "python -c 'import rusteze; env = rusteze.Env(seed=42); obs = env.reset(); obs, r, d = env.step({\"forward\": True}); print(f\"Reward: {r}, Done: {d}\")'", rusteze_dir),
        ("Multi-environment", "python -c 'import rusteze; multi = rusteze.MultiRustezeEnv(num_envs=2, seed=42); print(f\"Envs: {multi.num_envs()}\")'", rusteze_dir),
        ("Rust clippy", "cargo clippy --all-targets -- -D warnings", core_dir),
    ]
    
    results = []
    for name, command, cwd in tests:
        success = run_test(name, command, cwd)
        results.append((name, success))
    
    # Summary
    print("\n" + "="*60)
    print("Test Summary")
    print("="*60)
    
    passed = sum(1 for _, success in results if success)
    total = len(results)
    
    for name, success in results:
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"{status}: {name}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n🎉 All tests passed! Ready for release.")
        return 0
    else:
        print(f"\n⚠️  {total - passed} test(s) failed. Please fix before release.")
        return 1

if __name__ == "__main__":
    sys.exit(main())

