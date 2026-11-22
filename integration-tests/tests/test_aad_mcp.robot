*** Settings ***
Documentation    Integration tests for the Unified In-Process AAD MCP Server (Spec 45)
Library          RequestsLibrary
Library          Collections
Suite Setup      Create Session    backend    ${BE_BASE_URL}
Suite Teardown   Delete All Sessions

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Test Unified AAD MCP - Initialize And Tools List
    [Documentation]    Verify /api/aad/v1/mcp responds to initialize and tools/list with full AAD capability schemas.
    ${init_payload}=    Create Dictionary    jsonrpc=2.0    id=1    method=initialize
    ${init_resp}=    POST On Session    backend    /api/aad/v1/mcp    json=${init_payload}
    Should Be Equal As Integers    ${init_resp.status_code}    200
    ${init_json}=    Set Variable    ${init_resp.json()}
    Should Be Equal As Strings    ${init_json["result"]["serverInfo"]["name"]}    aad-be-mcp

    ${list_payload}=    Create Dictionary    jsonrpc=2.0    id=2    method=tools/list
    ${list_resp}=    POST On Session    backend    /api/aad/v1/mcp    json=${list_payload}
    Should Be Equal As Integers    ${list_resp.status_code}    200
    ${tools}=    Set Variable    ${list_resp.json()["result"]["tools"]}

    ${tool_names}=    Create List
    FOR    ${tool}    IN    @{tools}
        Append To List    ${tool_names}    ${tool["name"]}
        Dictionary Should Contain Key    ${tool}    inputSchema
    END

    List Should Contain Value    ${tool_names}    list_agents
    List Should Contain Value    ${tool_names}    search_agents
    List Should Contain Value    ${tool_names}    list_skills
    List Should Contain Value    ${tool_names}    list_traits
    List Should Contain Value    ${tool_names}    list_mcp_servers
    List Should Contain Value    ${tool_names}    search_agent_context
    List Should Contain Value    ${tool_names}    search_knowledge
    List Should Contain Value    ${tool_names}    list_benches
    List Should Contain Value    ${tool_names}    execute_agent

Test Unified AAD MCP - Call List Agents And List Traits
    [Documentation]    Verify live invocation of list_agents and list_traits via /api/aad/v1/mcp.
    ${empty_args}=    Create Dictionary
    ${params}=    Create Dictionary    name=list_agents    arguments=${empty_args}
    ${payload}=    Create Dictionary    jsonrpc=2.0    id=3    method=tools/call    params=${params}
    ${resp}=    POST On Session    backend    /api/aad/v1/mcp    json=${payload}
    Should Be Equal As Integers    ${resp.status_code}    200
    ${body}=    Set Variable    ${resp.json()}
    Should Be Equal    ${body["result"]["isError"]}    ${False}

    ${trait_params}=    Create Dictionary    name=list_traits    arguments=${empty_args}
    ${trait_payload}=    Create Dictionary    jsonrpc=2.0    id=4    method=tools/call    params=${trait_params}
    ${trait_resp}=    POST On Session    backend    /api/aad/v1/mcp    json=${trait_payload}
    Should Be Equal As Integers    ${trait_resp.status_code}    200
    ${trait_body}=    Set Variable    ${trait_resp.json()}
    Should Be Equal    ${trait_body["result"]["isError"]}    ${False}
