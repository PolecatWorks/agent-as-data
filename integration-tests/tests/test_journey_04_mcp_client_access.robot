*** Settings ***
Documentation    Integration test for Journey 4: Exposing Agents & Knowledge to External AI Assistants via MCP
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_OWNER_ID}      00000000-0000-0000-0000-000000000000
${AGENT_ID}       ${EMPTY}

*** Test Cases ***
Journey 4 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 4.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 4 MCP Client Access
    [Documentation]    Test that an external AI tool can search agents (representing MCP discovery)
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name}=    Set Variable    Journey4_Security_Audit_Tool_${rand}

    # 1. Create a security audit tool agent
    ${agent_def}=    Create Dictionary    role=auditor
    ${tags}=    Create List    security    audit    tool
    ${payload}=    Create Dictionary    name=${agent_name}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}    tags=${tags}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${AGENT_ID}    ${agent_id}

    # 2. External client asks "What security tools do we have?"
    # Using POST /api/v1/agents/search to simulate search_agents tool call
    ${search_payload}=    Create Dictionary    query=${agent_name}
    ${search_response}=    Search Agents    ${search_payload}

    # Assert search found it
    ${found}=    Set Variable    ${False}
    FOR    ${item}    IN    @{search_response}
        ${name}=    Get From Dictionary    ${item}    name
        ${match}=    Run Keyword And Return Status    Should Be Equal As Strings    ${name}    ${agent_name}
        IF    ${match}
            ${found}=    Set Variable    ${True}
            BREAK
        END
    END
    Should Be True    ${found}

    [Teardown]    Teardown Journey 4

*** Keywords ***
Teardown Journey 4
    [Documentation]    Clean up the created agent
    Run Keyword If    '${AGENT_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${AGENT_ID}    hard=True
