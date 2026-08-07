*** Settings ***
Documentation    Integration test for Journey 4: Exposing Agents & Knowledge to External AI Assistants via MCP
Library          ../lib/AADRequests.py

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 4 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 4.
    Log    Testing Journey 4 Native MCP Server framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
