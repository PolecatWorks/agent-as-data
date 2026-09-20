*** Settings ***
Documentation    Integration test for HaMS Prometheus Telemetry & Tokio Runtime Metrics
Library          RequestsLibrary
Library          String

*** Variables ***
${HAMS_BASE_URL}    http://localhost:8079

*** Test Cases ***
Verify HaMS Metrics Endpoint Responds With Baseline And Tokio Metrics
    [Documentation]    Verify that /hams/metrics returns 200 OK and contains both baseline app_info and tokio-metrics runtime telemetry.
    Create Session    hams_session    ${HAMS_BASE_URL}
    ${response}=    GET On Session    hams_session    /hams/metrics    expected_status=200
    ${body}=    Set Variable    ${response.text}

    # Verify baseline app_info telemetry
    Should Contain    ${body}    app_info
    Should Contain    ${body}    name="aad-be-container"

    # Verify tokio-metrics runtime telemetry
    Should Contain    ${body}    tokio_workers_count
    Should Contain    ${body}    tokio_live_tasks_count
