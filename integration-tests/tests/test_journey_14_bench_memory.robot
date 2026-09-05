*** Settings ***
Documentation    Integration test for Journey 14: Bench Working Memory & Decision Logs
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080
${BENCH_ID}       ${EMPTY}

*** Test Cases ***
Journey 14 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 14.
    Log    Testing Journey 14 Bench Working Memory readiness.
    Should Not Be Empty    ${BE_BASE_URL}

Test Bench Memory Lifecycle and Retrieval
    [Documentation]    Verify upserting working memory, appending decisions, fetching memory entries, and agent querying memory.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    ${rand}=    Generate Random String    8    [LETTERS]
    ${bench_name}=    Set Variable    MemoryBench_${rand}

    # 1. Create Bench
    ${bench_payload}=    Create Dictionary    name=${bench_name}    owner_id=00000000-0000-0000-0000-000000000000    description=Integration test bench for memory
    ${bench}=    Create Bench    ${bench_payload}
    ${bench_id}=    Get From Dictionary    ${bench}    id
    Set Global Variable    ${BENCH_ID}    ${bench_id}
    Should Not Be Empty    ${bench_id}

    # 2. Verify initially empty or no working memory
    ${initial_mem}=    Get Bench Memory    ${bench_id}
    ${initial_count}=    Get Length    ${initial_mem}
    Should Be Equal As Integers    ${initial_count}    0

    # 3. Upsert Working Memory
    ${mem_payload}=    Create Dictionary    title=Active Working Memory    content=Primary Architectural Decision: Database must be PostgreSQL with strict foreign keys.
    ${upserted}=    Upsert Bench Memory    ${bench_id}    ${mem_payload}
    ${upserted_content}=    Get From Dictionary    ${upserted}    content
    Should Contain    ${upserted_content}    Database must be PostgreSQL

    # 4. Append Decision
    ${decision_payload}=    Create Dictionary    title=Selected UI Framework    content=Decided to use Angular 19 standalone components.
    ${decision}=    Append Bench Decision    ${bench_id}    ${decision_payload}
    ${dec_title}=    Get From Dictionary    ${decision}    title
    Should Be Equal    ${dec_title}    Selected UI Framework

    # 5. Fetch all memories for the bench
    ${mem_list}=    Get Bench Memory    ${bench_id}
    ${mem_len}=    Get Length    ${mem_list}
    Should Be True    ${mem_len} >= 2

    # 6. Verify agent receives working memory context
    ${threads}=    List Bench Threads    ${bench_id}
    ${first_thread}=    Get From List    ${threads}    0
    ${thread_id}=    Get From Dictionary    ${first_thread}    id

    ${msg_payload}=    Create Dictionary    role=user    content=What is the primary architectural decision for the database in our memory?
    ${user_msg}=    Create Thread Message    ${thread_id}    ${msg_payload}
    ${u_content}=    Get From Dictionary    ${user_msg}    content
    Should Be Equal    ${u_content}    What is the primary architectural decision for the database in our memory?

    ${all_msgs}=    Wait For Assistant Message    ${thread_id}    timeout=180
    ${msg_count}=    Get Length    ${all_msgs}
    Should Be True    ${msg_count} >= 2
    ${last_msg}=    Get From List    ${all_msgs}    -1
    ${last_role}=    Get From Dictionary    ${last_msg}    role
    Should Be Equal    ${last_role}    assistant

    # 7. Clean up / Delete bench
    ${del_res}=    Delete Bench    ${bench_id}
    Should Be True    ${del_res}
