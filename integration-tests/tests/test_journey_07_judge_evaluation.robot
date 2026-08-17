*** Settings ***
Documentation    Integration test for Journey 7: Probabilistic Unit Testing & LLM-as-a-Judge Evaluation in CI/CD
Library          ../lib/AADRequests.py
Library          Collections

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${TEST_AGENT_NAME}    Journey7_Judge_Agent
${TEST_OWNER_ID}      11111111-1111-1111-1111-111111111111

*** Test Cases ***
Journey 7 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 7.
    Log    Testing Journey 7 LLM-as-a-Judge Evaluation framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
    ${health}=    Check Health
    Log    Health check result: ${health}

Test LLM-as-a-Judge Evaluation Engine & Regression Blocker
    [Documentation]    Test that testing an agent evaluates correctly, bumps version on pass, and blocks on fail based on judge_threshold.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    # 1. Create agent with a high judge_threshold that will fail (mock score is 0.9)
    ${agent_def}=    Create Dictionary    role=tester
    ${payload}=    Create Dictionary    name=${TEST_AGENT_NAME}    owner_id=${TEST_OWNER_ID}    judge_threshold=${0.95}    agent_definition=${agent_def}
    ${agent}=    Create Agent    ${payload}
    ${agent_id}=    Get From Dictionary    ${agent}    id
    ${initial_version}=    Get From Dictionary    ${agent}    current_version
    Should Be Equal As Integers    ${initial_version}    1

    # 2. Test agent -> Should fail threshold (0.9 < 0.95), version should not bump
    ${input}=    Create Dictionary    prompt=hello
    ${test_case}=    Create Dictionary    input=${input}    rubric=Should say hello
    ${test_cases}=    Create List    ${test_case}
    ${test_payload}=    Create Dictionary    test_cases=${test_cases}

    ${test_response}=    Test Agent    ${agent_id}    ${test_payload}
    ${status}=    Get From Dictionary    ${test_response}    status
    ${version_bumped}=    Get From Dictionary    ${test_response}    version_bumped
    Should Be Equal    ${status}    regression_blocked
    Should Be Equal As Strings    ${version_bumped}    False

    # 3. Update agent with a lower judge_threshold that will pass
    ${agent_def_updated}=    Create Dictionary    role=tester updated
    ${update_payload}=    Create Dictionary    name=${TEST_AGENT_NAME}    owner_id=${TEST_OWNER_ID}    judge_threshold=${0.8}    agent_definition=${agent_def_updated}
    ${updated_agent}=    Update Agent    ${agent_id}    ${update_payload}
    ${updated_threshold}=    Get From Dictionary    ${updated_agent}    judge_threshold
    Should Be Equal As Numbers    ${updated_threshold}    0.8

    # 4. Test agent again -> Should pass threshold (0.9 >= 0.8), version should bump
    ${test_response2}=    Test Agent    ${agent_id}    ${test_payload}
    ${status2}=    Get From Dictionary    ${test_response2}    status
    ${version_bumped2}=    Get From Dictionary    ${test_response2}    version_bumped
    ${new_version}=    Get From Dictionary    ${test_response2}    new_version

    Should Be Equal    ${status2}    passed
    Should Be Equal As Strings    ${version_bumped2}    True
    Should Be Equal As Integers    ${new_version}    2

    # 5. Delete agent -> Should succeed and cascade delete all test suites, runs, and revisions
    ${delete_response}=    Delete Agent    ${agent_id}
    ${deleted_id}=    Get From Dictionary    ${delete_response}    id
    Should Be Equal As Strings    ${deleted_id}    ${agent_id}

