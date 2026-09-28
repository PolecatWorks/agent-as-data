*** Settings ***
Library         RequestsLibrary
Library         Collections
Library         String

Suite Setup     Setup API Client
Suite Teardown  Delete All Sessions

*** Variables ***
${BE_BASE_URL}  http://localhost:8080
${TEST_TOPIC}   MCP Integration Testing
${NODE_ID}      ${EMPTY}

*** Keywords ***
Setup API Client
    Create Session    backend    ${BE_BASE_URL}

*** Test Cases ***

Test Health Check
    [Documentation]    Test that the base API is running
    ${resp}=    GET On Session    backend    /health
    Should Be Equal As Strings    ${resp.status_code}    200

Test Knowledge MCP - Tools List
    [Documentation]    Verify we can list tools
    ${payload}=    Create Dictionary    jsonrpc=2.0    id=1    method=tools/list
    ${resp}=    POST On Session    backend    /api/v1/knowledge/mcp    json=${payload}
    Should Be Equal As Strings    ${resp.status_code}    200
    ${json}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${json}    result
    Dictionary Should Contain Key    ${json["result"]}    tools
    ${tools}=    Set Variable    ${json["result"]["tools"]}
    Length Should Be    ${tools}    6
    ${tool_names}=    Evaluate    [t["name"] for t in $tools]
    List Should Contain Value    ${tool_names}    search_knowledge
    List Should Contain Value    ${tool_names}    ingest_knowledge

Test Knowledge MCP - Ingest Node
    [Documentation]    Verify we can ingest a knowledge node via MCP
    ${rand}=    Generate Random String    6    [LOWER]
    ${topic}=    Set Variable    ${TEST_TOPIC} ${rand}
    ${args}=    Create Dictionary    topic=${topic}    title=Test MCP Node    content=This is some test content about MCP tools.
    ${params}=    Create Dictionary    name=ingest_knowledge    arguments=${args}
    ${payload}=    Create Dictionary    jsonrpc=2.0    id=2    method=tools/call    params=${params}

    ${resp}=    POST On Session    backend    /api/v1/knowledge/mcp    json=${payload}
    Should Be Equal As Strings    ${resp.status_code}    200
    ${json}=    Set Variable    ${resp.json()}
    ${content}=    Set Variable    ${json["result"]["content"][0]["text"]}
    ${content_json}=    Evaluate    json.loads($content)    json
    Dictionary Should Contain Key    ${content_json}    id
    ${node_id}=    Set Variable    ${content_json["id"]}
    Set Suite Variable    ${NODE_ID}    ${node_id}

Test Knowledge MCP - Read Node
    [Documentation]    Verify we can read the ingested node
    Skip If    '${NODE_ID}' == '${EMPTY}'
    ${args}=    Create Dictionary    id=${NODE_ID}
    ${params}=    Create Dictionary    name=read_knowledge    arguments=${args}
    ${payload}=    Create Dictionary    jsonrpc=2.0    id=3    method=tools/call    params=${params}

    ${resp}=    POST On Session    backend    /api/v1/knowledge/mcp    json=${payload}
    Should Be Equal As Strings    ${resp.status_code}    200
    ${json}=    Set Variable    ${resp.json()}
    ${content}=    Set Variable    ${json["result"]["content"][0]["text"]}
    ${content_json}=    Evaluate    json.loads($content)    json
    Should Be Equal As Strings    ${content_json["id"]}    ${NODE_ID}
    Should Be Equal As Strings    ${content_json["title"]}    Test MCP Node

Test Knowledge MCP - Delete Node
    [Documentation]    Verify we can delete the ingested node
    Skip If    '${NODE_ID}' == '${EMPTY}'
    ${args}=    Create Dictionary    id=${NODE_ID}
    ${params}=    Create Dictionary    name=delete_knowledge    arguments=${args}
    ${payload}=    Create Dictionary    jsonrpc=2.0    id=4    method=tools/call    params=${params}

    ${resp}=    POST On Session    backend    /api/v1/knowledge/mcp    json=${payload}
    Should Be Equal As Strings    ${resp.status_code}    200
    ${json}=    Set Variable    ${resp.json()}
    ${content}=    Set Variable    ${json["result"]["content"][0]["text"]}
    ${content_json}=    Evaluate    json.loads($content)    json
    Should Be Equal As Strings    ${content_json["id"]}    ${NODE_ID}
    Should Be Equal As Strings    ${content_json["title"]}    Test MCP Node
