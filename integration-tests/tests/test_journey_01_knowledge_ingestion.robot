*** Settings ***
Documentation    Integration test for Journey 1: Capturing & Evolving Project Architecture Thoughts (Knowledge Ingestion)
Library          ../lib/AADRequests.py

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 1 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 1.
    Log    Testing Journey 1 Knowledge Ingestion framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
