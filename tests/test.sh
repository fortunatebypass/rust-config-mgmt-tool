#!/bin/bash
echo "[test] Running tests..."

echo "[test] Running tests/config.yml"
if /app/target/debug/rust-config-mgmt-tool /app/tests/config.yml; then
  echo "[test] Success"
else
  echo "[test] Failure"
  exit 1
fi

echo "[test] Running curl test"
if curltest=$(curl http://localhost/test.html); then
  if [ "$curltest" == "test" ]; then
    echo "[test] Success"
  else
    echo "[test] Failure - result '$curltest' did not match 'test'"
    exit 1
  fi
else
  echo "[test] Failure"
  exit 1
fi

echo "[test] Complete. All tests pass"
