*** Settings ***
Documentation    Journey 21: Declarative CLI Hydration (`aad-be ctl`) for Traits, Tools, Skills, and Agents
Library          ../lib/AADRequests.py
Library          OperatingSystem
Library          Process
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}       http://localhost:8080
${AAD_CTL_API_URL}   ${BE_BASE_URL}/api/aad/v1
${REPO_ROOT}         ${CURDIR}${/}..${/}..
${MANIFESTS_DIR}     ${REPO_ROOT}${/}manifests${/}ecosystem
${VARS_LOCAL}        ${REPO_ROOT}${/}manifests${/}vars${/}local.yaml
${VARS_CLUSTER}      ${REPO_ROOT}${/}manifests${/}vars${/}cluster-dev.yaml
${VARS_ISTIO}        ${REPO_ROOT}${/}manifests${/}vars${/}istio-gateway.yaml
${AAD_BIN}           ${REPO_ROOT}${/}aad-be-container${/}target${/}debug${/}aad-be-container

*** Keywords ***
Run AAD Ctl
    [Arguments]    @{args}
    ${bin_exists}=    Run Keyword And Return Status    File Should Exist    ${AAD_BIN}
    IF    ${bin_exists}
        ${result}=    Run Process    ${AAD_BIN}    ctl    @{args}    stdout=PIPE    stderr=PIPE
    ELSE
        ${result}=    Run Process    cargo    run    --quiet    --    ctl    @{args}    cwd=${REPO_ROOT}${/}aad-be-container    stdout=PIPE    stderr=PIPE
    END
    Log    STDOUT:\n${result.stdout}
    Log    STDERR:\n${result.stderr}
    RETURN    ${result}

*** Test Cases ***
Journey 21a: Offline Validation of Ecosystem Manifest Pack Across All Env Presets
    [Documentation]    Validates manifests/ecosystem offline (--self-contained) across default, local, cluster-dev, and istio-gateway presets.
    ${res_default}=    Run AAD Ctl    validate    -f    ${MANIFESTS_DIR}    -R    --self-contained
    Should Be Equal As Integers    ${res_default.rc}    0
    Should Contain    ${res_default.stdout}    27 total resources (6 traits, 4 tools, 11 skills, 6 agents)

    ${res_local}=    Run AAD Ctl    --vars-file    ${VARS_LOCAL}    validate    -f    ${MANIFESTS_DIR}    -R    --self-contained
    Should Be Equal As Integers    ${res_local.rc}    0

    ${res_cluster}=    Run AAD Ctl    --vars-file    ${VARS_CLUSTER}    validate    -f    ${MANIFESTS_DIR}    -R    --self-contained
    Should Be Equal As Integers    ${res_cluster.rc}    0

    ${res_istio}=    Run AAD Ctl    --vars-file    ${VARS_ISTIO}    validate    -f    ${MANIFESTS_DIR}    -R    --self-contained
    Should Be Equal As Integers    ${res_istio.rc}    0

Journey 21b: Live Hydration, Idempotent Re-Apply, Get, and Portable Export Round-Trip
    [Documentation]    Hydrates the full ecosystem pack into a live AAD backend, verifies idempotency on second apply, and validates exported manifests offline.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live hydration checks

    # 1. Dry-run preview
    ${dry}=    Run AAD Ctl    --url    ${AAD_CTL_API_URL}    apply    -f    ${MANIFESTS_DIR}    -R    --allow-degraded-tools    --dry-run
    Should Be Equal As Integers    ${dry.rc}    0
    Should Contain    ${dry.stdout}    Dry-run complete

    # 2. First apply (hydration)
    ${first}=    Run AAD Ctl    --url    ${AAD_CTL_API_URL}    apply    -f    ${MANIFESTS_DIR}    -R    --allow-degraded-tools    --strict-traits
    Should Be Equal As Integers    ${first.rc}    0
    Should Contain    ${first.stdout}    Hydration complete

    # 3. Second apply (idempotency: 0 created, 27 updated)
    ${second}=    Run AAD Ctl    --url    ${AAD_CTL_API_URL}    apply    -f    ${MANIFESTS_DIR}    -R    --allow-degraded-tools    --strict-traits
    Should Be Equal As Integers    ${second.rc}    0
    Should Contain    ${second.stdout}    0 created, 27 updated

    # 4. Inspect resources via ctl get
    ${get_all}=    Run AAD Ctl    --url    ${AAD_CTL_API_URL}    get    all    --output    table
    Should Be Equal As Integers    ${get_all.rc}    0
    Should Contain    ${get_all.stdout}    SemanticSteward
    Should Contain    ${get_all.stdout}    data-engine-mcp
    Should Contain    ${get_all.stdout}    plan_semantic_join_path
    Should Contain    ${get_all.stdout}    PolecatWorks Ecosystem Orchestrator Agent

    # 5. Export to temporary multi-doc YAML and validate offline
    ${export_file}=    Set Variable    /tmp/aad-journey-21-export.yaml
    ${exp}=    Run AAD Ctl    --url    ${AAD_CTL_API_URL}    export    --file    ${export_file}
    Should Be Equal As Integers    ${exp.rc}    0
    File Should Exist    ${export_file}

    ${val_exp}=    Run AAD Ctl    validate    -f    ${export_file}
    Should Be Equal As Integers    ${val_exp.rc}    0
    Remove File    ${export_file}
