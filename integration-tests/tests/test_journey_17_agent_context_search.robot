*** Settings ***
Documentation    Integration test for Journey 17: Agent Context Search & Embeddings Synchronization
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}       http://localhost:8080
${TEST_OWNER_ID}     22222222-2222-2222-2222-222222222222
${AGENT_ID}          ${EMPTY}

*** Test Cases ***
Journey 17 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 17.
    Should Not Be Empty    ${BE_BASE_URL}

Test Agent Context Search Roundtrip
    [Documentation]    Create an agent with unique description and prompt, sync embeddings, and retrieve via /api/v1/agent-context/search.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name}=    Set Variable    Journey17_Agent_${rand}
    ${keyword}=       Set Variable    KubeMeshExpert_${rand}
    ${description}=   Set Variable    Specialized consultant for ${keyword} architecture and ingress routing.
    ${agent_def}=     Create Dictionary    role=infrastructure-consultant    specialty=${keyword}

    # 1. Create the Agent
    ${payload}=    Create Dictionary    name=${agent_name}    description=${description}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${AGENT_ID}    ${agent_id}
    Should Not Be Empty    ${agent_id}

    # 2. Synchronize Embeddings for the Agent
    ${sync_res}=    Sync Agent Embeddings    ${agent_id}
    ${status}=    Get From Dictionary    ${sync_res}    status
    Should Be Equal As Strings    ${status}    success
    ${count}=    Get From Dictionary    ${sync_res}    embeddings_created
    Should Be True    ${count} >= 2

    # 3. Search for the unique keyword via Agent Context Search
    ${results}=    Search Agent Context    query=${keyword}    depth=5
    ${result_count}=    Get Length    ${results}
    Should Be True    ${result_count} >= 1

    # 4. Verify the top result matches the agent created
    ${top_result}=    Get From List    ${results}    0
    ${entity_id}=    Get From Dictionary    ${top_result}    entity_id
    Should Be Equal As Strings    ${entity_id}    ${agent_id}
    ${matched_content}=    Get From Dictionary    ${top_result}    content
    Should Contain    ${matched_content}    ${keyword}
    ${res_name}=    Get From Dictionary    ${top_result}    name
    Should Be Equal As Strings    ${res_name}    ${agent_name}
    ${res_desc}=    Get From Dictionary    ${top_result}    description
    Should Be Equal As Strings    ${res_desc}    ${description}
    ${match_reason}=    Get From Dictionary    ${top_result}    match_reason
    Should Not Be Empty    ${match_reason}
    Should Not Contain    ${match_reason}    Semantic similarity

    [Teardown]    Cleanup Agent

*** Keywords ***
Cleanup Agent
    [Documentation]    Clean up the agent created during testing.
    Run Keyword And Ignore Error    Delete Agent    ${AGENT_ID}    hard=True
    Log    Cleaned up agent ${AGENT_ID}
