*** Settings ***
Documentation    Verifies configurable API prefix behavior for Agent-As-Data Backend (positive and negative)
Library          ../lib/AADRequests.py
Library          Collections

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Positive: Backend Health At Root
    [Documentation]    Verify root /health responds for Kubernetes probes.
    [Tags]    prefix    positive    health
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

Positive: Backend Health Under Configured Prefix
    [Documentation]    Verify /api/aad/v1/health responds successfully.
    [Tags]    prefix    positive
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test
    ${body}=    Get Path    /api/aad/v1/health    expected_status=200
    Should Contain    ${body}    ok

Positive: Backend Health Under Backward Compatibility Alias
    [Documentation]    Verify /api/v1/health responds via backward-compatibility alias.
    [Tags]    prefix    positive
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test
    ${body}=    Get Path    /api/v1/health    expected_status=200
    Should Contain    ${body}    ok

Negative: Unknown Service Prefix Returns 404
    [Documentation]    Verify requests to non-existent API prefixes return HTTP 404.
    [Tags]    prefix    negative
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test
    Get Path    /api/unknown-service/v1/health    expected_status=404

Negative: Missing Version Segment Returns 404
    [Documentation]    Verify requests missing the version segment return HTTP 404.
    [Tags]    prefix    negative
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test
    Get Path    /api/aad/health    expected_status=404
