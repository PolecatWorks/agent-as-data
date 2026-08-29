*** Settings ***
Documentation    Integration test for Journey 7: Probabilistic Unit Testing & LLM-as-a-Judge Evaluation in CI/CD
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_OWNER_ID}      11111111-1111-1111-1111-111111111111
${AGENT_ID}       ${EMPTY}

*** Test Cases ***
Journey 7 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 7.
    Log    Testing Journey 7 LLM-as-a-Judge Evaluation framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}

Test LLM-as-a-Judge Evaluation Engine & Regression Blocker
    [Documentation]    Test that testing an agent evaluates correctly, bumps version on pass, and blocks on fail based on judge_threshold.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${agent_name}=    Set Variable    Journey7_Judge_Agent_${rand}

    # 1. Create agent with a high judge_threshold that will fail (mock score is 0.9)
    ${agent_def}=    Create Dictionary    role=tester
    ${payload}=    Create Dictionary    name=${agent_name}    owner_id=${TEST_OWNER_ID}    judge_threshold=${0.95}    agent_definition=${agent_def}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    Set Global Variable    ${AGENT_ID}    ${agent_id}
    ${initial_version}=    Get From Dictionary    ${agent}    current_version
    Should Be Equal As Strings    ${initial_version}    1.0.0

    # 2. Test agent -> Should fail threshold (score < 0.95), version should not bump
    ${input}=    Create Dictionary    prompt=The sky is blue
    ${test_case}=    Create Dictionary    input=${input}    rubric=Must provide a detailed 500-word essay proving the sky is bright neon green with verifiable mathematical equations
    ${test_cases}=    Create List    ${test_case}
    ${test_payload}=    Create Dictionary    test_cases=${test_cases}

    ${test_response}=    Test Agent    ${agent_id}    ${test_payload}
    ${status}=    Get From Dictionary    ${test_response}    status
    ${version_bumped}=    Get From Dictionary    ${test_response}    version_bumped
    Should Be Equal    ${status}    regression_blocked
    Should Be Equal As Strings    ${version_bumped}    False

    # 3. Update agent with a lower judge_threshold that will pass
    ${agent_def_updated}=    Create Dictionary    role=tester updated
    ${update_payload}=    Create Dictionary    name=${agent_name}    owner_id=${TEST_OWNER_ID}    judge_threshold=${0.8}    agent_definition=${agent_def_updated}
    ${updated_agent}=    Update Agent    ${agent_id}    ${update_payload}
    ${updated_threshold}=    Get From Dictionary    ${updated_agent}    judge_threshold
    Should Be Equal As Numbers    ${updated_threshold}    0.8

    # 4. Test agent again with valid passing test case -> Should pass threshold (score >= 0.8), version should bump
    ${pass_input}=    Create Dictionary    prompt=Say hello
    ${pass_test_case}=    Create Dictionary    input=${pass_input}    rubric=Any greeting or friendly acknowledgment is completely acceptable and satisfactory.
    ${pass_test_cases}=    Create List    ${pass_test_case}
    ${pass_test_payload}=    Create Dictionary    test_cases=${pass_test_cases}

    ${test_response2}=    Test Agent    ${agent_id}    ${pass_test_payload}
    ${status2}=    Get From Dictionary    ${test_response2}    status
    ${version_bumped2}=    Get From Dictionary    ${test_response2}    version_bumped
    ${new_version}=    Get From Dictionary    ${test_response2}    new_version

    Should Be Equal    ${status2}    passed
    Should Be Equal As Strings    ${version_bumped2}    True
    Should Be Equal As Strings    ${new_version}    1.1.0

    # 5. Execute agent to create a referencing record in executions table
    ${exec_payload}=    Create Dictionary    prompt=run diagnostic checks. Reply in 1 word: OK.
    ${exec_response}=    Execute Agent    ${agent_id}    ${exec_payload}

    # 6. Soft Delete agent -> Should succeed and mark it archived
    ${delete_response}=    Delete Agent    ${agent_id}
    ${deleted_id}=    Get From Dictionary    ${delete_response}    id
    Should Be Equal As Strings    ${deleted_id}    ${agent_id}
    ${archived_at}=    Get From Dictionary    ${delete_response}    archived_at
    Should Not Be Equal    ${archived_at}    ${None}

    # 7. Attempt Hard Delete -> Should fail because agent has referencing revisions/test runs/executions
    Run Keyword And Expect Error    *    Delete Agent    ${agent_id}    hard=True

    [Teardown]    Teardown Journey 7

*** Keywords ***
Teardown Journey 7
    [Documentation]    Clean up the created agent, though it's soft deleted
    Run Keyword If    '${AGENT_ID}' != '${EMPTY}'    Run Keyword And Ignore Error    Delete Agent    ${AGENT_ID}    hard=True
