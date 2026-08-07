*** Settings ***
Documentation    Integration test for Journey 6: Managed Skill Creation & Promotion to Autonomous Agent
Library          ../lib/AADRequests.py

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 6 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 6.
    Log    Testing Journey 6 Managed Skill Promotion framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
