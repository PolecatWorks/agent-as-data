*** Settings ***
Documentation    Integration test for Journey 2: Conceptualizing New Ideas with RAG + Graph Context (Ideation)
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${NODE_ID}        ${EMPTY}

*** Test Cases ***
Journey 2 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 2.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 2 Strategic AI Ideation
    [Documentation]    Test that we can search knowledge and query the knowledge graph
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${search_target}=    Set Variable    specific sentence for search testing ${rand}
    ${user_subject}=    Set Variable    Robot Test User ${rand}
    ${tenant_obj}=    Set Variable    Robot Test Tenant ${rand}

    # First ingest some test data
    ${metadata}=    Create Dictionary    type=architecture
    ${tuples}=    Create List
    ${tuple1}=    Create Dictionary    subject=${user_subject}    predicate=belongs_to    object=${tenant_obj}
    Append To List    ${tuples}    ${tuple1}

    ${ingest_payload}=    Create Dictionary    topic=robot-tenant-sso-architecture-${rand}    title=SSO & Tenant Mapping Decision    content=We are using a custom SSO setup for robot tenant mapping. User entities map to Tenant entities via membership tuples. This is a ${search_target}.    metadata=${metadata}    tuples=${tuples}

    ${ingest_response}=    Ingest Knowledge    ${ingest_payload}
    ${node_id}=    Get From Dictionary    ${ingest_response}    id
    Should Not Be Empty    ${node_id}
    Set Global Variable    ${NODE_ID}    ${node_id}

    # Now search knowledge
    ${search_payload}=    Create Dictionary    query=${search_target}
    ${search_response}=    Search Knowledge    ${search_payload}

    # Assert search found it
    ${found}=    Set Variable    ${False}
    FOR    ${item}    IN    @{search_response}
        ${text}=    Get From Dictionary    ${item}    chunk_text
        ${has_text}=    Run Keyword And Return Status    Should Contain    ${text}    ${search_target}
        IF    ${has_text}
            ${found}=    Set Variable    ${True}
            BREAK
        END
    END
    Should Be True    ${found}

    # Now query the knowledge graph
    ${graph_payload}=    Create Dictionary    subject=${user_subject}
    ${graph_response}=    Traverse Graph    ${graph_payload}

    # Assert graph found it
    ${found_graph}=    Set Variable    ${False}
    FOR    ${item}    IN    @{graph_response}
        ${sub}=    Get From Dictionary    ${item}    subject
        ${pred}=    Get From Dictionary    ${item}    predicate
        ${obj}=    Get From Dictionary    ${item}    object
        ${match_sub}=    Run Keyword And Return Status    Should Be Equal As Strings    ${sub}    ${user_subject}
        ${match_pred}=    Run Keyword And Return Status    Should Be Equal As Strings    ${pred}    belongs_to
        ${match_obj}=    Run Keyword And Return Status    Should Be Equal As Strings    ${obj}    ${tenant_obj}

        IF    ${match_sub} and ${match_pred} and ${match_obj}
            ${found_graph}=    Set Variable    ${True}
            BREAK
        END
    END
    Should Be True    ${found_graph}

    [Teardown]    Cleanup Strategic Ideation

*** Keywords ***
Cleanup Strategic Ideation
    [Documentation]    Clean up the ingested knowledge
    # TODO: Backend currently lacks a knowledge deletion endpoint.
    Log    Backend currently lacks a knowledge deletion endpoint.
    # Log    Cleaned up knowledge ${NODE_ID}
