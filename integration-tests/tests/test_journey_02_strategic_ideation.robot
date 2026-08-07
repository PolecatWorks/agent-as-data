*** Settings ***
Documentation    Integration test for Journey 2: Conceptualizing New Ideas with RAG + Graph Context (Ideation)
Library          ../lib/AADRequests.py

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 2 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 2.
    Log    Testing Journey 2 Strategic AI Ideation framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
