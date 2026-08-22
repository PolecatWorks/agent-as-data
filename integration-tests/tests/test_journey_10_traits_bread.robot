*** Settings ***
Documentation    Integration test for Journey 10: Traits Specification BREAD/CRUD APIs
Library          ../lib/AADRequests.py
Library          Collections

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Journey 10 Preflight Verification
    [Documentation]    Verify Robot Framework integration harness is functional for Journey 10.
    Log    Testing Journey 10 Traits BREAD framework readiness.
    Should Not Be Empty    ${BE_BASE_URL}
    ${health}=    Check Health
    Log    Health check result: ${health}

Test Traits BREAD Flow
    [Documentation]    Test that we can create, read, update, list, and delete a Trait Contract, iterating through all of its core features.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping live test

    # 1. Create Trait with initial features
    ${reqs}=    Create List    Read access to source code    Static analysis execution permission
    ${invariants}=    Create List    MUST NEVER leak API credentials
    ${criteria}=    Create List    Zero false negatives
    ${tags}=    Create List    security    audit
    ${input_g}=    Create Dictionary    active_guardrails=${EMPTY}
    ${output_g}=    Create Dictionary    active_guardrails=${EMPTY}
    ${guardrails}=    Create Dictionary    input_guardrails=${input_g}    output_guardrails=${output_g}
    ${payload}=    Create Dictionary    name=RobotSecurityTrait    owner_id=00000000-0000-0000-0000-000000000000    description=Security trait defined by robot integration test    capability_requirements=${reqs}    behavioral_invariants=${invariants}    evaluation_criteria=${criteria}    tags=${tags}    guardrails=${guardrails}

    ${trait}=    Create Trait    ${payload}
    ${trait_id}=    Get From Dictionary    ${trait}    id
    Should Not Be Empty    ${trait_id}
    ${name}=    Get From Dictionary    ${trait}    name
    Should Be Equal    ${name}    RobotSecurityTrait
    ${version}=    Get From Dictionary    ${trait}    version
    Should Be Equal As Strings    ${version}    1.0.0
 
    # Verify initial features are saved correctly
    ${fetched}=    Get Trait    ${trait_id}
    ${fetched_reqs}=    Get From Dictionary    ${fetched}    capability_requirements
    List Should Contain Value    ${fetched_reqs}    Read access to source code
    List Should Contain Value    ${fetched_reqs}    Static analysis execution permission
    
    ${fetched_invariants}=    Get From Dictionary    ${fetched}    behavioral_invariants
    List Should Contain Value    ${fetched_invariants}    MUST NEVER leak API credentials
    
    ${fetched_criteria}=    Get From Dictionary    ${fetched}    evaluation_criteria
    List Should Contain Value    ${fetched_criteria}    Zero false negatives
    
    ${fetched_tags}=    Get From Dictionary    ${fetched}    tags
    List Should Contain Value    ${fetched_tags}    security
    List Should Contain Value    ${fetched_tags}    audit
 
    # 2. Iterate and Update features (adding new ones, removing/changing old ones)
    ${updated_reqs}=    Create List    Read access to source code    AST parsing access
    ${updated_invariants}=    Create List    MUST NEVER leak API credentials    MUST NOT use insecure libraries
    ${updated_criteria}=    Create List    Zero false negatives    At least 95% test coverage
    ${updated_tags}=    Create List    security    compliance
    ${new_input_g}=    Create Dictionary    active_guardrails=${EMPTY}
    ${new_output_g}=    Create Dictionary    active_guardrails=${EMPTY}
    ${updated_guardrails}=    Create Dictionary    input_guardrails=${new_input_g}    output_guardrails=${new_output_g}
    
    ${update_payload}=    Create Dictionary    name=RobotSecurityTrait    owner_id=00000000-0000-0000-0000-000000000000    description=Security trait defined by robot integration test - updated    capability_requirements=${updated_reqs}    behavioral_invariants=${updated_invariants}    evaluation_criteria=${updated_criteria}    tags=${updated_tags}    guardrails=${updated_guardrails}
    
    ${updated}=    Update Trait    ${trait_id}    ${update_payload}
    ${updated_description}=    Get From Dictionary    ${updated}    description
    Should Be Equal    ${updated_description}    Security trait defined by robot integration test - updated
    ${updated_version}=    Get From Dictionary    ${updated}    version
    Should Be Equal As Strings    ${updated_version}    1.1.0

    # Verify updated features on backend
    ${fetched2}=    Get Trait    ${trait_id}
    
    # Capability requirements should have AST parsing access, but NOT Static analysis execution permission
    ${fetched2_reqs}=    Get From Dictionary    ${fetched2}    capability_requirements
    List Should Contain Value    ${fetched2_reqs}    Read access to source code
    List Should Contain Value    ${fetched2_reqs}    AST parsing access
    List Should Not Contain Value    ${fetched2_reqs}    Static analysis execution permission
    
    # Behavioral Invariants should have both
    ${fetched2_invariants}=    Get From Dictionary    ${fetched2}    behavioral_invariants
    List Should Contain Value    ${fetched2_invariants}    MUST NEVER leak API credentials
    List Should Contain Value    ${fetched2_invariants}    MUST NOT use insecure libraries
    
    # Evaluation Criteria should have both
    ${fetched2_criteria}=    Get From Dictionary    ${fetched2}    evaluation_criteria
    List Should Contain Value    ${fetched2_criteria}    Zero false negatives
    List Should Contain Value    ${fetched2_criteria}    At least 95% test coverage
    
    # Tags should have compliance but NOT audit
    ${fetched2_tags}=    Get From Dictionary    ${fetched2}    tags
    List Should Contain Value    ${fetched2_tags}    security
    List Should Contain Value    ${fetched2_tags}    compliance
    List Should Not Contain Value    ${fetched2_tags}    audit

    # 3. List (Browse) and confirm presence
    ${traits_response}=    List Traits
    ${ids}=    Get From Dictionary    ${traits_response}    ids
    List Should Contain Value    ${ids}    ${trait_id}

    # 4. Delete Trait
    ${deleted_trait}=    Delete Trait    ${trait_id}
    ${deleted_name}=    Get From Dictionary    ${deleted_trait}    name
    Should Be Equal    ${deleted_name}    RobotSecurityTrait

    # 5. Verify Deleted Trait is missing (404)
    Run Keyword And Expect Error    *    Get Trait    ${trait_id}
