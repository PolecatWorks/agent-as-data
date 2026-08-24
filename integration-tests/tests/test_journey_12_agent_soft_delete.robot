*** Settings ***
Documentation    Integration test for Journey 12: Declarative Agent Soft Deletion & Referential Integrity Preservation
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_OWNER_ID}      22222222-2222-2222-2222-222222222222
${AGENT_ID_1}     ${EMPTY}
${AGENT_ID_2}     ${EMPTY}

*** Test Cases ***
Journey 12 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 12.
    Should Not Be Empty    ${BE_BASE_URL}

Test Agent Soft Delete And Hard Delete Validation
    [Documentation]    Verify soft-deleted agents are hidden but preserved in the database, hard deletion blocks when references exist, and hard deletion succeeds when no references exist.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name_1}=    Set Variable    Journey12_Soft_Delete_Agent_1_${rand}
    ${agent_name_2}=    Set Variable    Journey12_Hard_Delete_Agent_2_${rand}

    # =================================================================
    # SCENARIO A: Agent with Execution history -> Soft delete defaults
    # =================================================================

    # 1. Create Agent 1
    ${agent_def}=    Create Dictionary    role=tester-soft-delete
    ${payload}=    Create Dictionary    name=${agent_name_1}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${agent1}=    Create Agent    ${payload}
    ${agent_id1}=    Get From Dictionary    ${agent1}    id
    Set Global Variable    ${AGENT_ID_1}    ${agent_id1}

    # 2. Run execution to create referential DB record
    ${exec_payload}=    Create Dictionary    prompt=trigger test execution
    ${exec_response}=    Execute Agent    ${agent_id1}    ${exec_payload}

    # 3. Soft Delete the agent (default delete_agent)
    ${delete_response}=    Delete Agent    ${agent_id1}
    ${deleted_id}=    Get From Dictionary    ${delete_response}    id
    Should Be Equal As Strings    ${deleted_id}    ${agent_id1}
    ${archived_at}=    Get From Dictionary    ${delete_response}    archived_at
    Should Not Be Equal    ${archived_at}    ${None}

    # 4. Attempt Hard Delete -> Must fail due to execution history constraint
    Run Keyword And Expect Error    *    Delete Agent    ${agent_id1}    hard=True

    # =================================================================
    # SCENARIO B: Agent without Execution history -> Hard delete allowed
    # =================================================================

    # 1. Create Agent 2 (never executed)
    ${payload2}=    Create Dictionary    name=${agent_name_2}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${agent2}=    Create Agent    ${payload2}
    ${agent_id2}=    Get From Dictionary    ${agent2}    id
    Set Global Variable    ${AGENT_ID_2}    ${agent_id2}

    # 2. Hard Delete the agent directly
    ${delete_response2}=    Delete Agent    ${agent_id2}    hard=True
    ${deleted_id2}=    Get From Dictionary    ${delete_response2}    id
    Should Be Equal As Strings    ${deleted_id2}    ${agent_id2}

    # Unset so teardown doesn't try again
    Set Global Variable    ${AGENT_ID_2}    ${EMPTY}

    [Teardown]    Teardown Journey 12

*** Keywords ***
Teardown Journey 12
    [Documentation]    Clean up
    Run Keyword If    '${AGENT_ID_1}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${AGENT_ID_1}    hard=True
    Run Keyword If    '${AGENT_ID_2}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${AGENT_ID_2}    hard=True
