*** Settings ***
Documentation    Integration tests for the 'aad-be ctl' CLI tool.
Library          Process
Library          String
Library          OperatingSystem

*** Variables ***
${MANIFEST_FILE}    /tmp/test-manifest.yaml

*** Test Cases ***
CLI Help Test
    [Documentation]    Test that the CLI tool executes and prints help text.
    [Tags]             local_only
    ${cargo_dir}=      Normalize Path    ${CURDIR}/../../aad-be-container
    ${exists}=         Run Keyword And Return Status    Directory Should Exist    ${cargo_dir}
    Pass Execution If  not ${exists}     CLI tests requiring cargo source directory are skipped in container environments
    ${result}=         Run Process    cargo    run    --bin    aad-be-container    --    ctl    --help    cwd=${cargo_dir}
    Should Be Equal As Integers    ${result.rc}    0
    Should Contain    ${result.stdout}    Manage resources declaratively via API
