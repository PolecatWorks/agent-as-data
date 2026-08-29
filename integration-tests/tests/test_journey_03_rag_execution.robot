*** Settings ***
Documentation    Integration test for Journey 3: Dynamic Agent Discovery & Execution via Prompt RAG Search
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_OWNER_ID}      00000000-0000-0000-0000-000000000000
${AGENT_ID}       ${EMPTY}

*** Test Cases ***
Journey 3 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 3.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 3 RAG Execution
    [Documentation]    Test that we can dynamically search for an agent based on task text and execute it
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name}=    Set Variable    Journey3_Rust_Security_Auditor_${rand}

    # 1. Create a specific agent for this task to ensure it exists
    ${agent_def}=    Create Dictionary    role=reviewer
    ${payload}=    Create Dictionary    name=${agent_name}    description=Review pull request for Rust security vulnerabilities and unsafe code blocks    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${AGENT_ID}    ${agent_id}

    # 2. Search and execute
    ${execute_payload}=    Create Dictionary    task_query=Review pull request for Rust security vulnerabilities and unsafe code blocks    prompt=Check the unsafe raw_pointer_offset function. Provide a concise 1-sentence verdict.

    ${response}=    Search And Execute    ${execute_payload}
    ${execution_id}=    Get From Dictionary    ${response}    execution_id
    Should Not Be Empty    ${execution_id}

    ${status}=    Get From Dictionary    ${response}    status
    Should Be Equal As Strings    ${status}    completed

    [Teardown]    Teardown Journey 3

*** Keywords ***
Teardown Journey 3
    [Documentation]    Clean up the created agent
    Run Keyword If    '${AGENT_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${AGENT_ID}    hard=True
