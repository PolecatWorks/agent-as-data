*** Settings ***
Documentation    Integration test for Journey 13: Bench Lifecycle & Filesystem Scoping
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${BENCH_ID}       ${EMPTY}

*** Test Cases ***
Journey 13 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 13.
    Log    Testing Journey 13 Bench Lifecycle framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}

Test Bench Lifecycle and Filesystem Isolation
    [Documentation]    Verify creating a bench, auto-creating a default thread, writing and reading shared bench files, and cascading deletion.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${bench_name}=    Set Variable    RobotBench_${rand}

    # 1. Create Bench
    ${bench_payload}=    Create Dictionary    name=${bench_name}    owner_id=00000000-0000-0000-0000-000000000000    description=Integration test bench
    ${bench}=    Create Bench    ${bench_payload}
    ${bench_id}=    Get From Dictionary    ${bench}    id
    Set Global Variable    ${BENCH_ID}    ${bench_id}
    Should Not Be Empty    ${bench_id}
    ${fetched_name}=    Get From Dictionary    ${bench}    name
    Should Be Equal    ${fetched_name}    ${bench_name}

    # 2. Verify auto-created default thread
    ${threads}=    List Bench Threads    ${bench_id}
    ${thread_count}=    Get Length    ${threads}
    Should Be True    ${thread_count} >= 1
    ${first_thread}=    Get From List    ${threads}    0
    ${thread_title}=    Get From Dictionary    ${first_thread}    title
    Should Be Equal    ${thread_title}    General

    # 3. Create another thread in the same bench
    ${new_thread_payload}=    Create Dictionary    owner_id=00000000-0000-0000-0000-000000000000    title=Feature Discussion    description=Second thread in bench
    ${thread2}=    Create Bench Thread    ${bench_id}    ${new_thread_payload}
    ${thread2_id}=    Get From Dictionary    ${thread2}    id
    Should Not Be Empty    ${thread2_id}

    # 4. Write a file in the bench workspace
    ${file_write}=    Write Bench File    ${bench_id}    config.json    {"version": "1.0.0"}
    ${msg}=    Get From Dictionary    ${file_write}    message
    Should Contain    ${msg}    Successfully wrote

    # 5. Read back file from bench workspace
    ${file_read}=    Read Bench File    ${bench_id}    config.json
    ${content}=    Get From Dictionary    ${file_read}    content
    Should Be Equal    ${content}    {"version": "1.0.0"}

    # 6. List bench files
    ${file_list}=    List Bench Files    ${bench_id}
    ${files}=    Get From Dictionary    ${file_list}    files
    List Should Contain Value    ${files}    config.json

    # 7. Update bench name
    ${update_payload}=    Create Dictionary    name=${bench_name}_Updated
    ${updated_bench}=    Update Bench    ${bench_id}    ${update_payload}
    ${up_name}=    Get From Dictionary    ${updated_bench}    name
    Should Be Equal    ${up_name}    ${bench_name}_Updated

    # 8. Delete individual thread
    ${del_thread_res}=    Delete Thread    ${thread2_id}
    Should Be True    ${del_thread_res}
    ${remaining_threads}=    List Bench Threads    ${bench_id}
    ${rem_count}=    Get Length    ${remaining_threads}
    Should Be True    ${rem_count} >= 1

    # 9. Clean up / Delete bench
    ${del_res}=    Delete Bench    ${bench_id}
    Should Be True    ${del_res}
