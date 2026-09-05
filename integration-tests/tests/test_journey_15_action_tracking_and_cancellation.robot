*** Settings ***
Documentation    Integration test for Journey 15: Persistent Action Tracking & Distributed Cancellation
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${BENCH_ID}       ${EMPTY}

*** Test Cases ***
Journey 15 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 15.
    Log    Testing Journey 15 Action Tracking and Cancellation framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}

Test Action Tracking and Completion
    [Documentation]    Verify an active run is created in PostgreSQL upon user prompt, tracks status, and completes.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${bench_name}=    Set Variable    ActionBench_${rand}

    # 1. Create Bench and Thread
    ${bench_payload}=    Create Dictionary    name=${bench_name}    owner_id=00000000-0000-0000-0000-000000000000
    ${bench}=    Create Bench    ${bench_payload}
    ${bench_id}=    Get From Dictionary    ${bench}    id
    Set Global Variable    ${BENCH_ID}    ${bench_id}

    ${threads}=    List Bench Threads    ${bench_id}
    ${thread}=    Get From List    ${threads}    0
    ${thread_id}=    Get From Dictionary    ${thread}    id

    # 2. Dispatch user message (Asynchronous 202)
    ${msg_payload}=    Create Dictionary    role=user    content=Hello assistant, please report our workspace file status.
    ${user_msg}=    Create Thread Message    ${thread_id}    ${msg_payload}
    ${run_id}=    Get From Dictionary    ${user_msg}    run_id
    Should Not Be Empty    ${run_id}

    # 3. Verify Active Run is discoverable
    ${runs}=    List Thread Runs    ${thread_id}
    ${run_count}=    Get Length    ${runs}
    Should Be True    ${run_count} >= 1

    # 4. Wait for background task completion
    ${all_msgs}=    Wait For Assistant Message    ${thread_id}    timeout=180
    ${msg_count}=    Get Length    ${all_msgs}
    Should Be True    ${msg_count} >= 2

    # 5. Verify run is no longer active
    ${active_run}=    Get Active Thread Run    ${thread_id}
    Should Be Equal    ${active_run}    ${None}

Test Distributed Cancellation and Pre-Tool Halting
    [Documentation]    Verify that cancelling an active run updates PostgreSQL, halts execution, omits tool side-effects, and injects cancellation notice.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${threads}=    List Bench Threads    ${BENCH_ID}
    ${thread}=    Get From List    ${threads}    0
    ${thread_id}=    Get From Dictionary    ${thread}    id

    # 1. Dispatch action requesting file creation
    ${msg_payload}=    Create Dictionary    role=user    content=Please create a new file named unwanted_cancelled_file.txt
    ${user_msg}=    Create Thread Message    ${thread_id}    ${msg_payload}
    ${run_id}=    Get From Dictionary    ${user_msg}    run_id
    Should Not Be Empty    ${run_id}

    # 2. Immediately cancel active run
    ${cancel_res}=    Cancel Active Thread Run    ${thread_id}
    ${cancel_status}=    Get From Dictionary    ${cancel_res}    status
    Should Be Equal    ${cancel_status}    cancelled

    # 3. Verify active run is immediately cleared
    ${active_run}=    Get Active Thread Run    ${thread_id}
    Should Be Equal    ${active_run}    ${None}

    # 4. Verify system cancellation message is present
    ${all_msgs}=    Get Thread Messages    ${thread_id}
    ${last_msg}=    Get From List    ${all_msgs}    -1
    ${last_role}=    Get From Dictionary    ${last_msg}    role
    ${last_content}=    Get From Dictionary    ${last_msg}    content
    Should Be Equal    ${last_role}    system
    Should Contain    ${last_content}    Action cancelled by user

    # 5. Verify file was NOT created
    ${files_res}=    List Bench Files    ${BENCH_ID}
    ${files}=    Get From Dictionary    ${files_res}    files
    List Should Not Contain Value    ${files}    unwanted_cancelled_file.txt

    # 6. Clean up / Delete bench
    ${del_res}=    Delete Bench    ${BENCH_ID}
    Should Be True    ${del_res}
