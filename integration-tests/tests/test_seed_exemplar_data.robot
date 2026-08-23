*** Settings ***
Documentation    Seed database with exemplar data (Traits, Tools, Skills, Agents)
Library          ../lib/AADRequests.py
Library          Collections
Library          String

*** Variables ***
${BE_BASE_URL}    http://localhost:8080

*** Test Cases ***
Seed Exemplar Data
    [Documentation]    Seeds the database with exemplar data extracted from the running instance.
    ${health}=    Check Health
    Pass Execution If    not ${health}    Backend is offline - skipping seeding
    
    # Empty guardrails
    ${empty_list}=    Create List
    ${empty_dict}=    Create Dictionary
    ${g_active}=    Create Dictionary    active_guardrails=${empty_list}
    ${empty_guardrails}=    Create Dictionary    input_guardrails=${g_active}    output_guardrails=${g_active}
    
    # 1. Create Trait: SecurityAudit
    ${audit_tags}=    Create List    security    audit
    ${audit_reqs}=    Create List
    ${audit_invariants}=    Create List
    ${audit_criteria}=    Create List
    ${trait1_payload}=    Create Dictionary    name=SecurityAudit    owner_id=00000000-0000-0000-0000-000000000000    description=Run a security audit checking for vulnerabilities and bad configurations.    capability_requirements=${audit_reqs}    behavioral_invariants=${audit_invariants}    evaluation_criteria=${audit_criteria}    tags=${audit_tags}    guardrails=${empty_guardrails}
    ${trait1}=    Create Trait    ${trait1_payload}
    ${trait1_id}=    Get From Dictionary    ${trait1}    id

    # 1b. Create Trait: PenTest
    ${pt_tags}=    Create List    security    test
    ${pt_reqs}=    Create List
    ${pt_invariants}=    Create List
    ${pt_criteria}=    Create List
    ${trait2_payload}=    Create Dictionary    name=PenTest    owner_id=00000000-0000-0000-0000-000000000000    description=Run a Security Penetration test to assess if known vulnerabilities are exploitable in the deployment.    capability_requirements=${pt_reqs}    behavioral_invariants=${pt_invariants}    evaluation_criteria=${pt_criteria}    tags=${pt_tags}    guardrails=${empty_guardrails}
    ${trait2}=    Create Trait    ${trait2_payload}
    ${trait2_id}=    Get From Dictionary    ${trait2}    id

    # 2. Register Tool: test-mcp-server-fixed
    ${tool_tags}=    Create List
    ${tool_config}=    Create Dictionary    url=http://localhost:3000/sse    tags=${tool_tags}    description=${EMPTY}
    ${tool_schema_tool1}=    Create Dictionary    name=search_agents    description=RAG search for matching agents
    ${tool_schema_tool2}=    Create Dictionary    name=execute_agent    description=Run agent with payload
    ${tool_schema_tools}=    Create List    ${tool_schema_tool1}    ${tool_schema_tool2}
    ${tool_schema_prompts}=    Create List
    ${tool_schema_resources}=    Create List
    ${tool_schema}=    Create Dictionary    tools=${tool_schema_tools}    prompts=${tool_schema_prompts}    resources=${tool_schema_resources}
    ${tool_payload}=    Create Dictionary    server_name=test-mcp-server-fixed    transport_type=sse    endpoint_config=${tool_config}    schema=${tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${tool1}=    Register Tool    ${tool_payload}
    ${tool1_id}=    Get From Dictionary    ${tool1}    id

    # 3. Create Skill: Quick
    ${quick_tags}=    Create List    security
    ${quick_tool_deps}=    Create List    ${tool1_id}
    ${quick_traits}=    Create List
    ${quick_def}=    Set Variable    \# Busy\n\nGet busy and jsut do stuff.
    ${quick_payload}=    Create Dictionary    name=Quick    owner_id=00000000-0000-0000-0000-000000000000    description=i do a lot of stuff and try to do many things.    tags=${quick_tags}    definition=${quick_def}    tool_dependencies=${quick_tool_deps}    implements_traits=${quick_traits}
    ${skill1}=    Create Skill    ${quick_payload}
    ${skill1_id}=    Get From Dictionary    ${skill1}    id

    # 3b. Create Skill: Useless
    ${useless_tags}=    Create List
    ${useless_tool_deps}=    Create List
    ${useless_traits}=    Create List    PenTest
    ${useless_def}=    Set Variable    \# Do very little
    ${useless_payload}=    Create Dictionary    name=Useless    owner_id=00000000-0000-0000-0000-000000000000    description=A simple skills    tags=${useless_tags}    definition=${useless_def}    tool_dependencies=${useless_tool_deps}    implements_traits=${useless_traits}
    ${skill2}=    Create Skill    ${useless_payload}
    ${skill2_id}=    Get From Dictionary    ${skill2}    id

    # 4. Create Agent: Journey7_Judge_Agent
    ${j7_def_str}=    Set Variable    {"role": "tester updated"}
    ${agent1_payload}=    Create Dictionary    name=Journey7_Judge_Agent    owner_id=11111111-1111-1111-1111-111111111111    description=${EMPTY}    agent_definition=${j7_def_str}    tags=${empty_list}    implements_traits=${empty_list}    skill_dependencies=${empty_list}    tool_dependencies=${empty_list}
    ${agent1}=    Create Agent    ${agent1_payload}

    # 4b. Create Agent: Journey12_Soft_Delete_Agent_1
    ${j12_def_str}=    Set Variable    {"role": "tester-soft-delete"}
    ${agent2_payload}=    Create Dictionary    name=Journey12_Soft_Delete_Agent_1    owner_id=22222222-2222-2222-2222-222222222222    description=${EMPTY}    agent_definition=${j12_def_str}    tags=${empty_list}    implements_traits=${empty_list}    skill_dependencies=${empty_list}    tool_dependencies=${empty_list}
    ${agent2}=    Create Agent    ${agent2_payload}

    # 4c. Create Agent: BasicSecurityTest
    ${bst_tags}=    Create List    ben    security    code
    ${bst_traits}=    Create List    SecurityAudit
    ${bst_skills}=    Create List    ${skill2_id}
    ${bst_tools}=    Create List    ${tool1_id}
    ${bst_def}=    Set Variable    \# Vicious Test\n\nHaving access to the code make a malitious attack using code and run it against the deployed systems.
    ${agent3_payload}=    Create Dictionary    name=BasicSecurityTest    owner_id=00000000-0000-0000-0000-000000000000    description=Create a simple assessment of the codebase and penetration test of the applications having prior knowledge of the codebase.    agent_definition=${bst_def}    tags=${bst_tags}    implements_traits=${bst_traits}    skill_dependencies=${bst_skills}    tool_dependencies=${bst_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${agent3}=    Create Agent    ${agent3_payload}

    Log    Successfully seeded exemplar data!
