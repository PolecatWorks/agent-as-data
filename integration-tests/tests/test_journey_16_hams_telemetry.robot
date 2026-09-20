*** Settings ***
Documentation    Integration test for HaMS Prometheus Telemetry & Tokio Runtime Metrics
Library          ../lib/AADRequests.py
Library          String

*** Test Cases ***
Verify Backend HaMS Metrics Endpoint Responds With Baseline And Tokio Metrics
    [Documentation]    Verify that backend /hams/metrics returns 200 OK and contains both baseline app_info and tokio-metrics runtime telemetry.
    ${metrics}=    Get Hams Metrics    service=backend
    Should Contain    ${metrics}    app_info
    Should Contain    ${metrics}    name="aad-be-container"
    Should Contain    ${metrics}    tokio_workers_count
    Should Contain    ${metrics}    tokio_live_tasks_count

Verify MCP HaMS Metrics Endpoint Responds With Baseline And Tokio Metrics If Online
    [Documentation]    Verify MCP /hams/metrics returns baseline app_info and tokio-metrics if service is online.
    ${is_ready}=    Get Hams Health    service=mcp    endpoint=ready
    Pass Execution If    not ${is_ready}    MCP server is offline - skipping live test
    ${metrics}=    Get Hams Metrics    service=mcp
    Should Contain    ${metrics}    app_info
    Should Contain    ${metrics}    name="agent-as-data-mcp"
    Should Contain    ${metrics}    tokio_workers_count
    Should Contain    ${metrics}    tokio_live_tasks_count
