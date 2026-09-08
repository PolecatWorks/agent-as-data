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
    ${result}=    Run Process    cargo    run    --bin    aad-be-container    --    ctl    --help    cwd=${CURDIR}/../../aad-be-container
    Should Be Equal As Integers    ${result.rc}    0
    Should Contain    ${result.stdout}    Manage resources declaratively via API
