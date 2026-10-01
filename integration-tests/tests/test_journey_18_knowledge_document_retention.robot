*** Settings ***
Documentation    Integration test for Journey 18: Knowledge Markdown Document Retention & Bidirectional Concept Provenance
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${DOC_NODE_ID}    ${EMPTY}
@{CHILD_NODE_IDS}

*** Test Cases ***
Journey 18 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 18.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 18 Markdown Document Ingestion With Retention & Provenance
    [Documentation]    Test that importing a markdown document retains the complete source document, child concepts, and bidirectional tuples.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${doc_topic}=    Set Variable    architecture-${rand}
    ${doc_title}=    Set Variable    Enterprise Architecture Blueprint ${rand}
    ${doc_content}=    Set Variable    \# Enterprise Architecture Blueprint ${rand}\n\nThis core document defines our hybrid storage system.\n\n### Vector Embeddings\nVector chunks are stored in pgvector.\n\n### Graph Store\nTuples are stored in knowledge_tuples.

    ${empty_tags}=    Create List
    ${doc_payload}=    Create Dictionary
    ...    topic=${doc_topic}
    ...    title=${doc_title}
    ...    description=High-level architecture specification for data persistence.
    ...    tags=${empty_tags}
    ...    content=${doc_content}

    ${concept1}=    Create Dictionary
    ...    topic=${doc_topic}
    ...    title=Vector Embeddings Subsystem ${rand}
    ...    description=pgvector chunk storage
    ...    tags=${empty_tags}
    ...    content=Vector chunks are stored in pgvector.

    ${concept2}=    Create Dictionary
    ...    topic=${doc_topic}
    ...    title=Graph Tuples Subsystem ${rand}
    ...    description=knowledge_tuples relation storage
    ...    tags=${empty_tags}
    ...    content=Tuples are stored in knowledge_tuples.

    ${concepts_list}=    Create List    ${concept1}    ${concept2}

    ${import_payload}=    Create Dictionary
    ...    document=${doc_payload}
    ...    concepts=${concepts_list}
    ...    create_tuples=${TRUE}

    ${res}=    Import Document    ${import_payload}

    # Verify Document Retention
    ${document}=    Get From Dictionary    ${res}    document
    ${doc_id}=    Get From Dictionary    ${document}    id
    Should Not Be Empty    ${doc_id}
    Set Global Variable    ${DOC_NODE_ID}    ${doc_id}

    ${doc_meta}=    Get From Dictionary    ${document}    metadata
    ${is_source_doc}=    Get From Dictionary    ${doc_meta}    is_source_document
    Should Be True    ${is_source_doc}

    # Verify Concepts and Provenance
    ${created_concepts}=    Get From Dictionary    ${res}    concepts
    ${concepts_len}=    Get Length    ${created_concepts}
    Should Be Equal As Integers    ${concepts_len}    2

    FOR    ${c}    IN    @{created_concepts}
        ${c_id}=    Get From Dictionary    ${c}    id
        Append To List    ${CHILD_NODE_IDS}    ${c_id}
        ${c_meta}=    Get From Dictionary    ${c}    metadata
        ${source_id}=    Get From Dictionary    ${c_meta}    source_document_id
        Should Be Equal As Strings    ${source_id}    ${doc_id}
    END

    ${tuples_count}=    Get From Dictionary    ${res}    tuples_created
    Should Be True    ${tuples_count} >= 2

    # Verify Derived Concepts Endpoint
    ${derived}=    Get Derived Concepts    ${doc_id}
    ${derived_len}=    Get Length    ${derived}
    Should Be Equal As Integers    ${derived_len}    2

    [Teardown]    Cleanup Ingested Document And Concepts

*** Keywords ***
Cleanup Ingested Document And Concepts
    [Documentation]    Clean up all ingested document and concept nodes
    FOR    ${child_id}    IN    @{CHILD_NODE_IDS}
        Run Keyword And Ignore Error    Delete Knowledge    ${child_id}
    END
    Run Keyword And Ignore Error    Delete Knowledge    ${DOC_NODE_ID}
    Log    Cleaned up document ${DOC_NODE_ID} and concepts
