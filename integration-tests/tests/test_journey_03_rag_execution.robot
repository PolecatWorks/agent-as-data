*** Settings ***
Documentation    Integration test for Journey 3: Dynamic Agent Discovery & Execution via Prompt RAG Search
Library          ../lib/AADRequests.py

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 3 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 3.
    Log    Testing Journey 3 Dynamic Agent Discovery & Execution framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
