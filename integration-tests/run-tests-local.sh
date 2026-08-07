#!/bin/bash
set -e

# Local test runner for robot tests against local dev services.
# Expects:
#   - Backend running on localhost:8080
#   - Frontend running on localhost:4200
#   - Database running on localhost:5432 (make compose-db-up)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

LOCAL_BE_URL="${LOCAL_BE_URL:-http://localhost:8080}"
LOCAL_FE_URL="${LOCAL_FE_URL:-http://localhost:4200}"

REPORT_DIR="${SCRIPT_DIR}/reports"
rm -rf "${REPORT_DIR}"
mkdir -p "${REPORT_DIR}"

RAW_TEST_PATH="${1:-${SCRIPT_DIR}/tests}"

if [[ "${RAW_TEST_PATH}" = /* ]]; then
    TEST_PATH="${RAW_TEST_PATH}"
else
    TEST_PATH="$(pwd)/${RAW_TEST_PATH}"
fi

# Detect robot executable (system or venv)
if command -v robot > /dev/null 2>&1; then
    ROBOT_CMD="robot"
elif [ -f "/Users/bengreene/Development/polecatworks/sward-warden/.venv/bin/robot" ]; then
    ROBOT_CMD="/Users/bengreene/Development/polecatworks/sward-warden/.venv/bin/robot"
else
    echo "Error: robot command not found."
    exit 1
fi

echo "=============================================="
echo " Agent-As-Data Robot Tests - Local Runner"
echo "=============================================="
echo "Backend URL:  ${LOCAL_BE_URL}"
echo "Frontend URL: ${LOCAL_FE_URL}"
echo "Test Path:    ${TEST_PATH}"
echo "Report Dir:   ${REPORT_DIR}"
echo "=============================================="

echo ""
echo "Running pre-flight checks..."

if curl -sf "${LOCAL_BE_URL}/health" > /dev/null 2>&1; then
    echo "  ✓ Backend is responding at ${LOCAL_BE_URL}"
else
    echo "  ! Backend is NOT responding at ${LOCAL_BE_URL} (Skipping live BE tests if stubbed)"
fi

if curl -sf "${LOCAL_FE_URL}" > /dev/null 2>&1; then
    echo "  ✓ Frontend is responding at ${LOCAL_FE_URL}"
else
    echo "  ! Frontend is NOT responding at ${LOCAL_FE_URL} (Skipping live FE tests if stubbed)"
fi

echo ""
echo "Starting robot tests..."
echo ""

"${ROBOT_CMD}" \
    --variable BE_BASE_URL:${LOCAL_BE_URL} \
    --variable FE_BASE_URL:${LOCAL_FE_URL} \
    --loglevel DEBUG \
    -d "${REPORT_DIR}" \
    "${TEST_PATH}"

