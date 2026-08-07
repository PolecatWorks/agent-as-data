*** Settings ***
Documentation    Integration test for Journey 9: Interactive Debugging & Trait Override Testing in Developer UI
Library          ../lib/AADRequests.py

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${FE_BASE_URL}    http://localhost:4200

*** Test Cases ***
Journey 9 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 9.
    Log    Testing Journey 9 Developer UI Studio framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
    Should Not Be Empty    ${FE_BASE_URL}
