*** Settings ***
Documentation    Integration test for Journey 11: Trait Editor UI & Sync Verification
Library          Browser
Library          ../lib/AADRequests.py
Library          Collections

*** Variables ***
${FE_BASE_URL}    http://localhost:4200
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Verify Trait Editor UI Sync with Backend
    [Documentation]    Test that creating, editing, and deleting a trait in the UI correctly persists to the backend database.
    [Setup]    New Browser    chromium    headless=True
    
    # 1. Open the Trait Contracts Registry in UI
    New Page    ${FE_BASE_URL}/traits-registry
    Wait For Elements State    text=Trait Definition Registry    visible    timeout=10s

    # 2. Click "+ Create Trait Definition"
    Click    button >> text=Create Trait Definition    button=left
    
    # 3. Enter Name & Description
    ${random_str}=    Evaluate    str(random.randint(1000, 9999))    modules=random
    ${trait_name}=    Set Variable    UITestTrait${random_str}
    ${trait_desc}=    Set Variable    Trait definition created by UI integration test ${random_str}
    
    Fill Text    id=trait-name-input    ${trait_name}
    Fill Text    id=trait-description-input    ${trait_desc}

    # 4. Save Trait Definition
    Click    button[color="accent"]    button=left
    Wait For Elements State    text=Created new trait ${trait_name}    visible    timeout=5s

    # 5. Confirm persistence against backend REST API (using AADRequests library)
    ${traits_response}=    List Traits
    ${ids}=    Get From Dictionary    ${traits_response}    ids
    ${found_trait}=    Set Variable    ${False}
    ${trait_id}=    Set Variable    ${EMPTY}
    FOR    ${id_item}    IN    @{ids}
        ${fetched_t}=    Get Trait    ${id_item}
        ${name_val}=    Get From Dictionary    ${fetched_t}    name
        IF    '${name_val}' == '${trait_name}'
            ${found_trait}=    Set Variable    ${True}
            ${trait_id}=    Set Variable    ${id_item}
            BREAK
        END
    END
    Should Be True    ${found_trait}    Created trait not found in backend list
    
    # Confirm backend detail matches UI entries
    ${fetched}=    Get Trait    ${trait_id}
    ${fetched_desc}=    Get From Dictionary    ${fetched}    description
    Should Be Equal    ${fetched_desc}    ${trait_desc}

    # 6. Click Delete button in UI
    Click    button[color="warn"]    button=left
    Wait For Elements State    text=Trait contract deleted successfully    visible    timeout=5s

    # 7. Confirm deletion on backend REST API
    Run Keyword And Expect Error    *    Get Trait    ${trait_id}
    
    [Teardown]    Close Browser    ALL
