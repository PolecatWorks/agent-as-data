*** Settings ***
Documentation    Integration test for Journey 6: Managed Skill Creation & Promotion to Autonomous Agent
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_OWNER_ID}      00000000-0000-0000-0000-000000000000
${SKILL_ID}       ${EMPTY}
${PROMOTED_AGENT_ID}  ${EMPTY}

*** Test Cases ***
Journey 6 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 6.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 6 Skill Promotion
    [Documentation]    Test that we can create a skill and promote it to an agent
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${skill_name}=    Set Variable    Journey6_JSON_Log_Formatter_${rand}

    # 1. Create a skill
    ${tags}=    Create List    logging    formatter
    ${payload}=    Create Dictionary    name=${skill_name}    owner_id=${TEST_OWNER_ID}    description=Formats JSON logs    prompt=Format the incoming log string into standard JSON    tags=${tags}

    ${skill}=    Create Skill    ${payload}
    ${skill_id}=    Get From Dictionary    ${skill}    id
    Set Global Variable    ${SKILL_ID}    ${skill_id}

    # 2. Promote the skill
    ${promoted_agent}=    Promote Skill    ${SKILL_ID}
    ${agent_id}=    Get From Dictionary    ${promoted_agent}    id
    Set Global Variable    ${PROMOTED_AGENT_ID}    ${agent_id}

    ${name}=    Get From Dictionary    ${promoted_agent}    name
    Should Be Equal As Strings    ${name}    ${skill_name}

    [Teardown]    Teardown Journey 6

*** Keywords ***
Teardown Journey 6
    [Documentation]    Clean up the created skill and agent
    Run Keyword If    '${SKILL_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Skill    ${SKILL_ID}
    Run Keyword If    '${PROMOTED_AGENT_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${PROMOTED_AGENT_ID}    hard=True
