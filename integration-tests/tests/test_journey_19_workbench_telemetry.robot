*** Settings ***
Documentation    Integration tests for Workbench Thread Run Telemetry
Library          ../lib/AADRequests.py
Library          String

*** Variables ***
${BE_BASE_URL}  http://localhost:8080
${BENCH_ID}     ${EMPTY}
${THREAD_ID}    ${EMPTY}
${MESSAGE_ID}   ${EMPTY}

*** Test Cases ***
Create Bench And Thread For Telemetry Testing
    [Documentation]    Setup a bench and a thread to test the run telemetry.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${bench_name}=    Generate Random String    8    [LOWER]
    ${payload}=    Create Dictionary    name=${bench_name}    owner_id=00000000-0000-0000-0000-000000000000
    ${bench}=    Create Bench    ${payload}
    Set Suite Variable    ${BENCH_ID}    ${bench["id"]}

    ${payload}=    Create Dictionary    title=Telemetry Test Thread    owner_id=00000000-0000-0000-0000-000000000000
    ${thread}=    Create Bench Thread    ${BENCH_ID}    ${payload}
    Set Suite Variable    ${THREAD_ID}    ${thread["id"]}

Test Create Message Populates Originating Message ID and Model Telemetry
    [Documentation]    Sending a message creates a thread_run with originating_message_id and model.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${payload}=    Create Dictionary    role=user    content=Say hello
    ${response}=    Create Thread Message    ${THREAD_ID}    ${payload}
    Set Suite Variable    ${MESSAGE_ID}    ${response["id"]}
    ${run_id}=    Set Variable    ${response["run_id"]}

    Wait For Assistant Message    ${THREAD_ID}    timeout=30

    ${runs}=    List Thread Runs    ${THREAD_ID}
    Should Not Be Empty    ${runs}
    ${first_run}=    Set Variable    ${runs[0]}
    Should Be Equal As Strings    ${first_run["originating_message_id"]}    ${MESSAGE_ID}
    Should Be Equal As Strings    ${first_run["status"]}    completed
    Should Not Be Empty    ${first_run["model"]}
    Should Not Be Empty    ${first_run["duration_ms"]}

*** Keywords ***
Teardown Test Data
    [Documentation]    Clean up bench if it was created
    Run Keyword And Ignore Error    Delete Bench    ${BENCH_ID}
