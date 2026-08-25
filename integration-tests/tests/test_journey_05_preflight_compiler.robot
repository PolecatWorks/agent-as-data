*** Settings ***
Documentation    Integration test for Journey 5: Pre-Flight Agent Network Compilation & Conceptual Validation
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_OWNER_ID}      00000000-0000-0000-0000-000000000000
${ROOT_AGENT_ID}  ${EMPTY}

*** Test Cases ***
Journey 5 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 5.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 5 Preflight Compiler
    [Documentation]    Test that we can compile an agent network
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name}=    Set Variable    Journey5_Code_Reviewer_Orchestrator_${rand}

    # 1. Create a root agent
    ${agent_def}=    Create Dictionary    role=orchestrator
    ${payload}=    Create Dictionary    name=${agent_name}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${ROOT_AGENT_ID}    ${agent_id}

    # 2. Compile the agent
    ${compile_payload}=    Create Dictionary    root_agent_id=${ROOT_AGENT_ID}
    ${compile_response}=    Compile Agent    ${compile_payload}

    ${status}=    Get From Dictionary    ${compile_response}    status
    Should Be Equal As Strings    ${status}    clean

    [Teardown]    Teardown Journey 5

*** Keywords ***
Teardown Journey 5
    [Documentation]    Clean up the created agent
    Run Keyword If    '${ROOT_AGENT_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${ROOT_AGENT_ID}    hard=True
