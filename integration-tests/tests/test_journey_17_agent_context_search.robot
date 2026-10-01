*** Settings ***
Documentation    Integration test for Journey 17: Agent Context Search, Natural Language Retrieval & UI Discovery
Library          Browser
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${FE_BASE_URL}       http://localhost:4200
${BE_BASE_URL}       http://localhost:8080
${TEST_OWNER_ID}     22222222-2222-2222-2222-222222222222
${AGENT_ID}          ${EMPTY}
${DEDUP_AGENT_ID}    ${EMPTY}

*** Test Cases ***
Journey 17 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 17.
    Should Not Be Empty    ${BE_BASE_URL}
    Should Not Be Empty    ${FE_BASE_URL}

Test Agent Context Search Roundtrip With Exact Keyword And Reverse References
    [Documentation]    Create an agent with unique description and prompt, verify immediate automated embedding indexing with reverse references, verify manual sync endpoint, verify update, and verify purge on delete.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name}=    Set Variable    Journey17_Agent_${rand}
    ${keyword}=       Set Variable    KubeMeshExpert_${rand}
    ${description}=   Set Variable    Specialized consultant for ${keyword} architecture and ingress routing.
    ${agent_def}=     Create Dictionary    role=infrastructure-consultant    specialty=${keyword}

    # 1. Create the Agent (Automated Embedding Sync via BREAD)
    ${payload}=    Create Dictionary    name=${agent_name}    description=${description}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${AGENT_ID}    ${agent_id}
    Should Not Be Empty    ${agent_id}

    # 2. Search immediately without manual sync - verify automated embedding indexing & reverse references
    ${results}=    Search Agent Context    query=${keyword}    depth=5
    ${result_count}=    Get Length    ${results}
    Should Be True    ${result_count} >= 1

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

    # Verify Reverse Reference Metadata
    ${origin_id}=    Get From Dictionary    ${top_result}    origin_id
    Should Be Equal As Strings    ${origin_id}    ${agent_id}
    ${origin_type}=    Get From Dictionary    ${top_result}    origin_type
    Should Be Equal As Strings    ${origin_type}    agents
    ${origin_uri}=    Get From Dictionary    ${top_result}    origin_uri
    Should Be Equal As Strings    ${origin_uri}    /agents/${agent_id}
    ${origin_name}=    Get From Dictionary    ${top_result}    origin_name
    Should Be Equal As Strings    ${origin_name}    ${agent_name}

    # 3. Synchronize Embeddings manually for the Agent (verify backwards-compatible CLI/test endpoint)
    ${sync_res}=    Sync Agent Embeddings    ${agent_id}
    ${status}=    Get From Dictionary    ${sync_res}    status
    Should Be Equal As Strings    ${status}    success
    ${count}=    Get From Dictionary    ${sync_res}    embeddings_created
    Should Be True    ${count} >= 2

    # 4. Update the Agent with a new keyword and verify automatic re-index
    ${updated_keyword}=    Set Variable    UpdatedMeshRouter_${rand}
    ${updated_desc}=    Set Variable    Updated controller for ${updated_keyword} protocols.
    ${update_payload}=    Create Dictionary    name=${agent_name}    description=${updated_desc}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${updated_agent}=    Update Agent    ${agent_id}    ${update_payload}

    ${updated_results}=    Search Agent Context    query=${updated_keyword}    depth=5
    ${updated_count}=    Get Length    ${updated_results}
    Should Be True    ${updated_count} >= 1
    ${updated_top}=    Get From List    ${updated_results}    0
    ${updated_entity_id}=    Get From Dictionary    ${updated_top}    entity_id
    Should Be Equal As Strings    ${updated_entity_id}    ${agent_id}

    # 5. Delete the Agent and verify purge from search results
    ${del_res}=    Delete Agent    ${agent_id}    hard=True
    Set Global Variable    ${AGENT_ID}    ${EMPTY}

    ${purged_results}=    Search Agent Context    query=${updated_keyword}    depth=5
    ${found_deleted}=    Set Variable    ${FALSE}
    FOR    ${res}    IN    @{purged_results}
        ${id_val}=    Get From Dictionary    ${res}    entity_id
        IF    '${id_val}' == '${agent_id}'
            ${found_deleted}=    Set Variable    ${TRUE}
        END
    END
    Should Not Be True    ${found_deleted}

    [Teardown]    Cleanup Agent    ${AGENT_ID}


Test Natural Language Sentence Search With Multi-Word Stems
    [Documentation]    Test that natural language queries with stop words and stems retrieve conceptually matching agents.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    # Query with stop words, stemming, and multiple domain concepts
    ${nl_query}=    Set Variable    i need an agent that can be used to design accounting systems
    ${results}=    Search Agent Context    query=${nl_query}    depth=5
    ${result_count}=    Get Length    ${results}
    Should Be True    ${result_count} >= 1

    # Top result should be FinancialAuditorAgent or FinancialControllerAgent
    ${top_result}=    Get From List    ${results}    0
    ${res_name}=    Get From Dictionary    ${top_result}    name
    Should Contain Any    ${res_name}    FinancialAuditorAgent    FinancialControllerAgent    ComplianceChecking

    ${res_desc}=    Get From Dictionary    ${top_result}    description
    Should Not Be Empty    ${res_desc}

    ${match_reason}=    Get From Dictionary    ${top_result}    match_reason
    Should Contain    ${match_reason}    Matched on entity
    Should Not Contain    ${match_reason}    Semantic similarity

    ${score}=    Get From Dictionary    ${top_result}    score
    Should Be True    ${score} >= 0.70

Test Whitespace And Trailing Newline Resiliency
    [Documentation]    Test that searches with leading/trailing newlines and spaces are sanitized properly.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${padded_query}=    Catenate    SEPARATOR=    \n    accounting    \n\n
    ${results}=    Search Agent Context    query=${padded_query}    depth=5
    ${result_count}=    Get Length    ${results}
    Should Be True    ${result_count} >= 1

    ${top_result}=    Get From List    ${results}    0
    ${res_name}=    Get From Dictionary    ${top_result}    name
    Should Contain Any    ${res_name}    FinancialAuditorAgent    ComplianceChecking

Test Entity Deduplication Across Multiple Matching Fields
    [Documentation]    Test that an entity matching on multiple fields is deduplicated and returned once.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name}=    Set Variable    DedupAgent_${rand}
    ${unique_term}=   Set Variable    MegaCoreRouter_${rand}
    ${description}=   Set Variable    Primary controller for ${unique_term} mesh network.
    ${agent_def}=     Create Dictionary    role=core-routing    details=${unique_term}

    ${payload}=    Create Dictionary    name=${agent_name}    description=${description}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${DEDUP_AGENT_ID}    ${agent_id}

    ${sync_res}=    Sync Agent Embeddings    ${agent_id}
    Should Be Equal As Strings    ${sync_res}[status]    success

    ${results}=    Search Agent Context    query=${unique_term}    depth=10
    ${matches_count}=    Set Variable    0
    FOR    ${res}    IN    @{results}
        ${id_val}=    Get From Dictionary    ${res}    entity_id
        IF    '${id_val}' == '${agent_id}'
            ${matches_count}=    Evaluate    ${matches_count} + 1
        END
    END
    # Must appear exactly once in the results list despite multiple fields matching
    Should Be Equal As Integers    ${matches_count}    1

    [Teardown]    Cleanup Agent    ${DEDUP_AGENT_ID}

Verify Agent Context Search UI And Navigation
    [Documentation]    End-to-end browser journey verifying textarea enter keydown, cards, name, reason, description, and navigation.
    [Setup]    New Browser    chromium    headless=True
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend/Frontend is offline - skipping live test

    # 1. Open Agent Context Search View
    New Page    ${FE_BASE_URL}/agent-context
    Wait For Elements State    text=Semantic Context Discovery    visible    timeout=10s

    # 2. Enter Natural Language Query and press Enter
    Fill Text    textarea[matInput]    i need an agent that can be used to design accounting systems
    Keyboard Key    press    Enter

    # 3. Verify Result Cards Appear
    Wait For Elements State    mat-card:has-text("FinancialAuditorAgent")    visible    timeout=10s

    # 4. Assert name, match reason, and description are rendered
    ${card_text}=    Get Text    mat-card:has-text("FinancialAuditorAgent")
    Should Contain    ${card_text}    FinancialAuditorAgent
    Should Contain    ${card_text}    Matched on entity description
    Should Contain    ${card_text}    A detail-oriented accounting agent specialized in financial audits
    Should Not Contain    ${card_text}    Semantic similarity

    # 5. Click View Details and verify navigation to detail page
    Click    mat-card:has-text("FinancialAuditorAgent") >> [data-testid="view-details-btn"]
    Wait For Elements State    input[placeholder="Search agents by name..."]    visible    timeout=10s
    ${current_url}=    Get Url
    Should Contain    ${current_url}    /agents/
    [Teardown]    Close Browser

Verify Sync Embeddings Button Removed From Registries
    [Documentation]    Verify the "Sync Embeddings" button has been completely removed from Agent and Skill registries.
    [Setup]    New Browser    chromium    headless=True
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend/Frontend is offline - skipping live test

    # 1. Visit Agent Registry
    New Page    ${FE_BASE_URL}/agents
    Wait For Elements State    text=New Agent    visible    timeout=10s
    ${agent_sync_btn_count}=    Get Element Count    button:has-text("Sync Embeddings")
    Should Be Equal As Integers    ${agent_sync_btn_count}    0

    # 2. Visit Skills Registry
    New Page    ${FE_BASE_URL}/skills
    Wait For Elements State    text=New Skill    visible    timeout=10s
    ${skill_sync_btn_count}=    Get Element Count    button:has-text("Sync Embeddings")
    Should Be Equal As Integers    ${skill_sync_btn_count}    0
    [Teardown]    Close Browser

*** Keywords ***
Cleanup Agent
    [Arguments]    ${id_to_delete}
    IF    '${id_to_delete}' != '${EMPTY}'
        Run Keyword And Ignore Error    Delete Agent    ${id_to_delete}    hard=True
        Log    Cleaned up agent ${id_to_delete}
    END
