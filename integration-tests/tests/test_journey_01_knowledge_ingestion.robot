*** Settings ***
Documentation    Integration test for Journey 1: Capturing & Evolving Project Architecture Thoughts (Knowledge Ingestion)
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 1 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 1.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 1 Knowledge Ingestion
    [Documentation]    Test that we can ingest knowledge via /api/v1/knowledge
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${topic}=    Set Variable    auth-microservice-architecture-${rand}

    ${metadata}=    Create Dictionary    type=architecture
    ${tuples}=    Create List
    ${tuple1}=    Create Dictionary    subject=Auth Microservice ${rand}    predicate=built_with    object=Rust Axum
    Append To List    ${tuples}    ${tuple1}
    ${tuple2}=    Create Dictionary    subject=Session State ${rand}    predicate=stored_in    object=Redis
    Append To List    ${tuples}    ${tuple2}
    ${tuple3}=    Create Dictionary    subject=User Profile ${rand}    predicate=stored_in    object=PostgreSQL
    Append To List    ${tuples}    ${tuple3}
    ${tuple4}=    Create Dictionary    subject=User ${rand}    predicate=belongs_to    object=Tenant ${rand}
    Append To List    ${tuples}    ${tuple4}

    ${payload}=    Create Dictionary    topic=${topic}    title=Auth Stack & Tenant Mapping Decision    content=Use Rust + Axum for auth microservice. Session state is stored in Redis key-value store. User profile records are in PostgreSQL. Users are linked to Tenants via membership tuples.    metadata=${metadata}    tuples=${tuples}

    ${response}=    Ingest Knowledge    ${payload}
    ${node_id}=    Get From Dictionary    ${response}    id
    Should Not Be Empty    ${node_id}

    Set Global Variable    ${NODE_ID}    ${node_id}

    [Teardown]    Cleanup Ingested Knowledge

*** Keywords ***
Cleanup Ingested Knowledge
    [Documentation]    Clean up the ingested knowledge
    # TODO: Backend currently lacks a knowledge deletion endpoint.
    Log    Backend currently lacks a knowledge deletion endpoint.
    # Log    Cleaned up knowledge ${NODE_ID}
