*** Settings ***
Documentation     Tests for Workbench Multi-turn Journeys, validating context retention and mocked tool execution.
Library           RequestsLibrary
Library           Collections
Library           String
Library           OperatingSystem

Suite Setup       Setup Test Suite
Suite Teardown    Teardown Test Suite

*** Variables ***
${API_URL}        http://localhost:3000/v1
${BENCH_ID}       ${EMPTY}
${THREAD_ID}      ${EMPTY}

*** Test Cases ***

Test Multi-Turn Journey 2: Brainstorming and File Writing
    [Documentation]    Validates that the agent can generate text in turn 1, and write specifically referenced context to a file in turn 2.

    # --- Setup Thread ---
    ${thread_payload}=    Create Dictionary    title=Brainstorming Journey    bench_id=${BENCH_ID}
    ${thread_resp}=    POST    ${API_URL}/threads    json=${thread_payload}
    Should Be Equal As Integers    ${thread_resp.status_code}    201
    ${thread}=    Set Variable    ${thread_resp.json()}
    Set Suite Variable    ${THREAD_ID}    ${thread['id']}

    # --- Turn 1: Brainstorming Request ---
    ${chat1_payload}=    Create Dictionary    message=Brainstorm 3 ideas for a new 'Task Management' feature. Give me 3 bullet points.
    ${chat1_resp}=    POST    ${API_URL}/threads/${THREAD_ID}/chat    json=${chat1_payload}
    Should Be Equal As Integers    ${chat1_resp.status_code}    200
    ${chat1_json}=    Set Variable    ${chat1_resp.json()}
    Should Contain    ${chat1_json['response']}    *

    # --- Turn 2: Write to file ---
    ${chat2_payload}=    Create Dictionary    message=I like the second one. Can you create a markdown file named feature_idea.md and document that specific point in detail?
    ${chat2_resp}=    POST    ${API_URL}/threads/${THREAD_ID}/chat    json=${chat2_payload}
    Should Be Equal As Integers    ${chat2_resp.status_code}    200

    # --- Validation: Check File Exists & Content ---
    ${fs_read_resp}=    GET    ${API_URL}/threads/${THREAD_ID}/fs/read/feature_idea.md
    Should Be Equal As Integers    ${fs_read_resp.status_code}    200
    ${file_content}=    Set Variable    ${fs_read_resp.text}
    Should Not Be Empty    ${file_content}

Test Multi-Turn Journey 1: Code Generation and Execution (Mocked)
    [Documentation]    Validates that the agent writes python code and attempts to use the (mocked) MCP execution tool.

    # --- Setup Thread ---
    ${thread_payload}=    Create Dictionary    title=Code Execution Journey    bench_id=${BENCH_ID}
    ${thread_resp}=    POST    ${API_URL}/threads    json=${thread_payload}
    Should Be Equal As Integers    ${thread_resp.status_code}    201
    ${thread}=    Set Variable    ${thread_resp.json()}
    Set Suite Variable    ${THREAD_ID}    ${thread['id']}

    # --- Turn 1: Write Code ---
    ${chat1_payload}=    Create Dictionary    message=Can you write a simple Python script called hello.py that prints 'Hello Workbench'?
    ${chat1_resp}=    POST    ${API_URL}/threads/${THREAD_ID}/chat    json=${chat1_payload}
    Should Be Equal As Integers    ${chat1_resp.status_code}    200

    # Check it wrote the file
    ${fs_read_resp}=    GET    ${API_URL}/threads/${THREAD_ID}/fs/read/hello.py
    Should Be Equal As Integers    ${fs_read_resp.status_code}    200
    Should Contain    ${fs_read_resp.text}    print

    # --- Turn 2: Run Code ---
    ${chat2_payload}=    Create Dictionary    message=Did you create it? Can you run it and tell me the output?
    ${chat2_resp}=    POST    ${API_URL}/threads/${THREAD_ID}/chat    json=${chat2_payload}
    Should Be Equal As Integers    ${chat2_resp.status_code}    200

    # We expect the agent to report the mocked execution output
    ${chat2_json}=    Set Variable    ${chat2_resp.json()}
    Should Contain    ${chat2_json['response']}    Hello Workbench

*** Keywords ***
Setup Test Suite
    # Create a Bench to hold the threads
    ${bench_payload}=    Create Dictionary    name=MultiTurn Journeys Bench
    ${bench_resp}=    POST    ${API_URL}/benches    json=${bench_payload}
    Run Keyword If    ${bench_resp.status_code} != 201    Fail    Failed to create Bench
    ${bench}=    Set Variable    ${bench_resp.json()}
    Set Suite Variable    ${BENCH_ID}    ${bench['id']}

Teardown Test Suite
    Run Keyword And Ignore Error    DELETE    ${API_URL}/benches/${BENCH_ID}
