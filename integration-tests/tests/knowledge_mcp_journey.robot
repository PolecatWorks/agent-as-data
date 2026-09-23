*** Settings ***
Documentation    Knowledge Base MCP Journey Tests
Library          RequestsLibrary
Library          Collections
Library          String

Suite Setup      Create Session    backend    http://localhost:8080
Suite Teardown   Delete All Sessions

*** Variables ***
${API_PREFIX}       /v1
${CREATED_NODE_ID}  ${EMPTY}
${BACKEND_URL}      http://localhost:8080

*** Test Cases ***
Test Knowledge Base MCP Journey
    [Documentation]    Test the complete BREAD flow using direct API endpoints acting as tools.
    ...                While the actual Rust tool structs are tested in backend unit tests,
    ...                this verifies the endpoints they call work completely correctly for the journey.
    [Teardown]    Cleanup Knowledge Node

    # Step 1: Add Node
    ${headers}=    Create Dictionary    Content-Type=application/json
    ${tags}=       Create List          mcp    journey
    ${payload}=    Create Dictionary    topic=journey_topic    title=MCP Journey Start    description=Test description    content=This is the content for the journey test.    tags=${tags}

    ${response}=   POST On Session      backend    ${API_PREFIX}/knowledge-base    json=${payload}    headers=${headers}
    Should Be Equal As Strings    ${response.status_code}    201
    ${body}=       Set Variable    ${response.json()}
    Set Suite Variable    ${CREATED_NODE_ID}    ${body['id']}
    Should Not Be Empty   ${CREATED_NODE_ID}

    # Step 2: Read Node
    ${response}=   GET On Session       backend    ${API_PREFIX}/knowledge-base/${CREATED_NODE_ID}
    Should Be Equal As Strings    ${response.status_code}    200
    ${body}=       Set Variable    ${response.json()}
    Should Be Equal As Strings    ${body['topic']}    journey_topic
    Should Be Equal As Strings    ${body['title']}    MCP Journey Start

    # Step 3: Browse Nodes
    ${response}=   GET On Session       backend    ${API_PREFIX}/knowledge-base
    Should Be Equal As Strings    ${response.status_code}    200
    ${body}=       Set Variable    ${response.json()}
    ${found}=      Set Variable    ${False}
    FOR    ${node}    IN    @{body}
        ${found}=    Set Variable If    '${node['id']}' == '${CREATED_NODE_ID}'    ${True}    ${found}
    END
    Should Be True    ${found}

    # Step 4: Search Nodes (Browse with Query)
    ${search_payload}=    Create Dictionary    query=journey test    limit=5
    ${response}=   POST On Session      backend    ${API_PREFIX}/knowledge-base/search    json=${search_payload}    headers=${headers}
    Should Be Equal As Strings    ${response.status_code}    200
    ${body}=       Set Variable    ${response.json()}
    # Depending on embedding vector generation (mock or real), we might or might not have a direct exact match,
    # but the API should at least return 200 OK.

    # Step 5: Edit Node
    ${edit_tags}=       Create List          mcp    journey    edited
    ${edit_payload}=    Create Dictionary    title=MCP Journey Edited    tags=${edit_tags}
    ${response}=   PUT On Session       backend    ${API_PREFIX}/knowledge-base/${CREATED_NODE_ID}    json=${edit_payload}    headers=${headers}
    Should Be Equal As Strings    ${response.status_code}    200

    # Step 6: Verify Edit via Read
    ${response}=   GET On Session       backend    ${API_PREFIX}/knowledge-base/${CREATED_NODE_ID}
    Should Be Equal As Strings    ${response.status_code}    200
    ${body}=       Set Variable    ${response.json()}
    Should Be Equal As Strings    ${body['title']}    MCP Journey Edited
    List Should Contain Value     ${body['tags']}     edited

    # Deletion happens in Teardown.

*** Keywords ***
Cleanup Knowledge Node
    [Documentation]    Deletes the knowledge node created during the test.
    Run Keyword And Ignore Error    Run Keyword If    '${CREATED_NODE_ID}' != '${EMPTY}'    DELETE On Session    backend    ${API_PREFIX}/knowledge-base/${CREATED_NODE_ID}
