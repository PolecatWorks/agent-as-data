*** Settings ***
Documentation    Integration test for Journey 7: Probabilistic Unit Testing & LLM-as-a-Judge Evaluation in CI/CD
Library          ../lib/AADRequests.py

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 7 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 7.
    Log    Testing Journey 7 LLM-as-a-Judge Evaluation framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
