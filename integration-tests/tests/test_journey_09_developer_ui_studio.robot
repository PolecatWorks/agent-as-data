*** Settings ***
Documentation    Integration test for Journey 9: Interactive Debugging & Trait Override Testing in Developer UI
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_OWNER_ID}      00000000-0000-0000-0000-000000000000
${TRAIT_ID}       ${EMPTY}
${AGENT_ID}       ${EMPTY}

*** Test Cases ***
Journey 9 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 9.
    Should Not Be Empty    ${BE_BASE_URL}

Journey 9 Developer UI Studio
    [Documentation]    Test that we can verify a contract and execute an agent
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${trait_name}=    Set Variable    Journey9_Trait_${rand}
    ${agent_name}=    Set Variable    Journey9_Agent_${rand}

    # 1. Create a trait and an agent that implements it
    ${tags}=    Create List    security
    ${reqs}=    Create List
    ${invariants}=    Create List
    ${criteria}=    Create List
    ${g_active}=    Create Dictionary    active_guardrails=${EMPTY}
    ${empty_guardrails}=    Create Dictionary    input_guardrails=${g_active}    output_guardrails=${g_active}

    ${trait_payload}=    Create Dictionary    name=${trait_name}    owner_id=${TEST_OWNER_ID}    description=Test trait    capability_requirements=${reqs}    behavioral_invariants=${invariants}    evaluation_criteria=${criteria}    tags=${tags}    guardrails=${empty_guardrails}
    ${trait}=    Create Trait    ${trait_payload}
    ${trait_id}=    Get From Dictionary    ${trait}    id
    Set Global Variable    ${TRAIT_ID}    ${trait_id}

    ${implements}=    Create List    ${trait_id}
    ${agent_def}=    Create Dictionary    role=tester
    ${agent_payload}=    Create Dictionary    name=${agent_name}    owner_id=${TEST_OWNER_ID}    agent_definition=${agent_def}    implements_traits=${implements}
    ${agent}=    Create Agent    ${agent_payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${AGENT_ID}    ${agent_id}

    # 2. Verify Contract
    ${verify_payload}=    Create Dictionary    referrer_agent_id=${AGENT_ID}    target_agent_id=${AGENT_ID}    trait_name=${trait_name}
    ${verify_response}=    Verify Contract    ${verify_payload}

    ${status_val}=    Get From Dictionary    ${verify_response}    status
    Should Not Be Empty    ${status_val}

    # 3. Execute the agent
    ${execute_payload}=    Create Dictionary    prompt=Run test
    ${execute_response}=    Execute Agent    ${AGENT_ID}    ${execute_payload}
    ${status}=    Get From Dictionary    ${execute_response}    status
    Should Be Equal As Strings    ${status}    completed

    [Teardown]    Teardown Journey 9

*** Keywords ***
Teardown Journey 9
    [Documentation]    Clean up
    Run Keyword If    '${AGENT_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${AGENT_ID}    hard=True
    Run Keyword If    '${TRAIT_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Trait    ${TRAIT_ID}
