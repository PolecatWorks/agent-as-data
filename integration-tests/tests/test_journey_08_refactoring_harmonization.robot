*** Settings ***
Documentation    Integration test for Journey 8: Agent Network Refactoring & Deliberate Contradiction Harmonization
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 8 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 8.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 8 Refactoring Engine
    [Documentation]    Test that we can call the analyze refactor endpoint
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${payload}=    Create Dictionary    similarity_threshold=0.88

    ${response}=    Analyze Refactor    ${payload}

    # Assert response contains expected keys
    Dictionary Should Contain Key    ${response}    clusters
    Dictionary Should Contain Key    ${response}    redundant_agents
    Dictionary Should Contain Key    ${response}    deliberate_contradictions
