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

    # 5. Developer Usage
    ${dev_trait_tags}=    Create List    developer    code    quality    review
    ${dev_trait_payload}=    Create Dictionary    name=CodeReview    owner_id=00000000-0000-0000-0000-000000000000    description=Extensive review of code structure, identifying anti-patterns, evaluating compliance with coding standards, and suggesting syntax or architectural improvements.    capability_requirements=${empty_list}    behavioral_invariants=${empty_list}    evaluation_criteria=${empty_list}    tags=${dev_trait_tags}    guardrails=${empty_guardrails}
    ${dev_trait}=    Create Trait    ${dev_trait_payload}
    ${dev_trait_id}=    Get From Dictionary    ${dev_trait}    id

    ${dev_tool_tags}=    Create List    git    developer
    ${dev_tool_config}=    Create Dictionary    url=http://localhost:3000/git-sse    tags=${dev_tool_tags}    description=Integrates with git to access repositories
    ${dev_tool_schema}=    Create Dictionary    tools=${empty_list}    prompts=${empty_list}    resources=${empty_list}
    ${dev_tool_payload}=    Create Dictionary    server_name=git-mcp-server    transport_type=sse    endpoint_config=${dev_tool_config}    schema=${dev_tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${dev_tool}=    Register Tool    ${dev_tool_payload}
    ${dev_tool_id}=    Get From Dictionary    ${dev_tool}    id

    ${dev_skill_tags}=    Create List    developer    refactoring
    ${dev_skill_tool_deps}=    Create List    ${dev_tool_id}
    ${dev_skill_traits}=    Create List    ${dev_trait_id}
    ${dev_skill_def}=    Set Variable    \# Code Refactoring\\n\\nApply modern design patterns to refactor legacy code, extract methods, reduce cyclomatic complexity, and ensure high test coverage.
    ${dev_skill_payload}=    Create Dictionary    name=Refactoring    owner_id=00000000-0000-0000-0000-000000000000    description=Advanced skill for modifying codebase layout and syntax to improve readability, maintainability, and performance without altering external behavior.    tags=${dev_skill_tags}    definition=${dev_skill_def}    tool_dependencies=${dev_skill_tool_deps}    implements_traits=${dev_skill_traits}
    ${dev_skill}=    Create Skill    ${dev_skill_payload}
    ${dev_skill_id}=    Get From Dictionary    ${dev_skill}    id

    ${dev_agent_tags}=    Create List    developer    senior
    ${dev_agent_traits}=    Create List    ${dev_trait_id}
    ${dev_agent_skills}=    Create List    ${dev_skill_id}
    ${dev_agent_tools}=    Create List    ${dev_tool_id}
    ${dev_agent_def}=    Set Variable    \# Senior Developer\\n\\nYou are a senior developer. Review code meticulously and use git tools to implement sweeping, high-quality refactoring changes.
    ${dev_agent_payload}=    Create Dictionary    name=SeniorDeveloperAgent    owner_id=00000000-0000-0000-0000-000000000000    description=An expert developer agent capable of autonomous code review, Git operations, and large-scale codebase refactoring to meet enterprise standards.    agent_definition=${dev_agent_def}    tags=${dev_agent_tags}    implements_traits=${dev_agent_traits}    skill_dependencies=${dev_agent_skills}    tool_dependencies=${dev_agent_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${dev_agent}=    Create Agent    ${dev_agent_payload}

    # 6. System Architecture
    ${arch_trait_tags}=    Create List    architecture    design    system
    ${arch_trait_payload}=    Create Dictionary    name=SystemDesign    owner_id=00000000-0000-0000-0000-000000000000    description=Assess and design robust, highly available, and scalable systems, including microservices architecture, data storage layers, and network topologies.    capability_requirements=${empty_list}    behavioral_invariants=${empty_list}    evaluation_criteria=${empty_list}    tags=${arch_trait_tags}    guardrails=${empty_guardrails}
    ${arch_trait}=    Create Trait    ${arch_trait_payload}
    ${arch_trait_id}=    Get From Dictionary    ${arch_trait}    id

    ${arch_tool_tags}=    Create List    diagrams    architecture
    ${arch_tool_config}=    Create Dictionary    url=http://localhost:3000/drawio-sse    tags=${arch_tool_tags}    description=Tool for reading and generating DrawIO architectural diagrams
    ${arch_tool_schema}=    Create Dictionary    tools=${empty_list}    prompts=${empty_list}    resources=${empty_list}
    ${arch_tool_payload}=    Create Dictionary    server_name=drawio-mcp-server    transport_type=sse    endpoint_config=${arch_tool_config}    schema=${arch_tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${arch_tool}=    Register Tool    ${arch_tool_payload}
    ${arch_tool_id}=    Get From Dictionary    ${arch_tool}    id

    ${arch_skill_tags}=    Create List    cloud    architecture
    ${arch_skill_tool_deps}=    Create List    ${arch_tool_id}
    ${arch_skill_traits}=    Create List    ${arch_trait_id}
    ${arch_skill_def}=    Set Variable    \# Cloud Architecture\\n\\nDesign cloud-native deployment solutions utilizing multi-region redundancy, serverless functions, and container orchestration platforms like Kubernetes.
    ${arch_skill_payload}=    Create Dictionary    name=CloudArchitecture    owner_id=00000000-0000-0000-0000-000000000000    description=Skill focused on translating business requirements into technical cloud infrastructure and creating comprehensive topological diagrams.    tags=${arch_skill_tags}    definition=${arch_skill_def}    tool_dependencies=${arch_skill_tool_deps}    implements_traits=${arch_skill_traits}
    ${arch_skill}=    Create Skill    ${arch_skill_payload}
    ${arch_skill_id}=    Get From Dictionary    ${arch_skill}    id

    ${arch_agent_tags}=    Create List    architect    system
    ${arch_agent_traits}=    Create List    ${arch_trait_id}
    ${arch_agent_skills}=    Create List    ${arch_skill_id}
    ${arch_agent_tools}=    Create List    ${arch_tool_id}
    ${arch_agent_def}=    Set Variable    \# Software Architect\\n\\nYou are a software architect. Plan system boundaries, define API contracts, and evaluate the trade-offs of various technology stacks.
    ${arch_agent_payload}=    Create Dictionary    name=SoftwareArchitectAgent    owner_id=00000000-0000-0000-0000-000000000000    description=A principal architect agent responsible for high-level system decisions, drafting technical blueprints, and evaluating framework suitability for large enterprises.    agent_definition=${arch_agent_def}    tags=${arch_agent_tags}    implements_traits=${arch_agent_traits}    skill_dependencies=${arch_agent_skills}    tool_dependencies=${arch_agent_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${arch_agent}=    Create Agent    ${arch_agent_payload}

    # 7. Business Analyst
    ${ba_trait_tags}=    Create List    business    requirements    analysis
    ${ba_trait_payload}=    Create Dictionary    name=RequirementsElicitation    owner_id=00000000-0000-0000-0000-000000000000    description=Techniques and capabilities for gathering, documenting, and managing business requirements, translating stakeholder needs into functional specifications.    capability_requirements=${empty_list}    behavioral_invariants=${empty_list}    evaluation_criteria=${empty_list}    tags=${ba_trait_tags}    guardrails=${empty_guardrails}
    ${ba_trait}=    Create Trait    ${ba_trait_payload}
    ${ba_trait_id}=    Get From Dictionary    ${ba_trait}    id

    ${ba_tool_tags}=    Create List    jira    agile
    ${ba_tool_config}=    Create Dictionary    url=http://localhost:3000/jira-sse    tags=${ba_tool_tags}    description=Atlassian Jira integration for managing issues, epics, and sprints
    ${ba_tool_schema}=    Create Dictionary    tools=${empty_list}    prompts=${empty_list}    resources=${empty_list}
    ${ba_tool_payload}=    Create Dictionary    server_name=jira-mcp-server    transport_type=sse    endpoint_config=${ba_tool_config}    schema=${ba_tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${ba_tool}=    Register Tool    ${ba_tool_payload}
    ${ba_tool_id}=    Get From Dictionary    ${ba_tool}    id

    ${ba_skill_tags}=    Create List    process    modeling
    ${ba_skill_tool_deps}=    Create List    ${ba_tool_id}
    ${ba_skill_traits}=    Create List    ${ba_trait_id}
    ${ba_skill_def}=    Set Variable    \# Process Modeling\\n\\nMap complex business processes and workflows, identifying inefficiencies, bottlenecks, and opportunities for automation and optimization.
    ${ba_skill_payload}=    Create Dictionary    name=ProcessModeling    owner_id=00000000-0000-0000-0000-000000000000    description=Ability to utilize standards like BPMN to document current-state and future-state operational models for business transformation projects.    tags=${ba_skill_tags}    definition=${ba_skill_def}    tool_dependencies=${ba_skill_tool_deps}    implements_traits=${ba_skill_traits}
    ${ba_skill}=    Create Skill    ${ba_skill_payload}
    ${ba_skill_id}=    Get From Dictionary    ${ba_skill}    id

    ${ba_agent_tags}=    Create List    analyst    business
    ${ba_agent_traits}=    Create List    ${ba_trait_id}
    ${ba_agent_skills}=    Create List    ${ba_skill_id}
    ${ba_agent_tools}=    Create List    ${ba_tool_id}
    ${ba_agent_def}=    Set Variable    \# Business Analyst\\n\\nYou act as a liaison between stakeholders and development teams, writing crisp user stories and maintaining the product backlog.
    ${ba_agent_payload}=    Create Dictionary    name=BusinessAnalystAgent    owner_id=00000000-0000-0000-0000-000000000000    description=Dedicated business analyst agent skilled in Agile methodology, requirement scoping, and backlog grooming, ensuring alignment with strategic objectives.    agent_definition=${ba_agent_def}    tags=${ba_agent_tags}    implements_traits=${ba_agent_traits}    skill_dependencies=${ba_agent_skills}    tool_dependencies=${ba_agent_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${ba_agent}=    Create Agent    ${ba_agent_payload}

    # 8. Financial Controller
    ${fc_trait_tags}=    Create List    finance    planning    control
    ${fc_trait_payload}=    Create Dictionary    name=FinancialPlanning    owner_id=00000000-0000-0000-0000-000000000000    description=Strategic analysis of financial operations including budgeting, forecasting, and the creation of financial controls to ensure fiscal responsibility.    capability_requirements=${empty_list}    behavioral_invariants=${empty_list}    evaluation_criteria=${empty_list}    tags=${fc_trait_tags}    guardrails=${empty_guardrails}
    ${fc_trait}=    Create Trait    ${fc_trait_payload}
    ${fc_trait_id}=    Get From Dictionary    ${fc_trait}    id

    ${fc_tool_tags}=    Create List    erp    finance
    ${fc_tool_config}=    Create Dictionary    url=http://localhost:3000/erp-sse    tags=${fc_tool_tags}    description=Enterprise Resource Planning (ERP) API connector for fetching real-time financial metrics
    ${fc_tool_schema}=    Create Dictionary    tools=${empty_list}    prompts=${empty_list}    resources=${empty_list}
    ${fc_tool_payload}=    Create Dictionary    server_name=erp-mcp-server    transport_type=sse    endpoint_config=${fc_tool_config}    schema=${fc_tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${fc_tool}=    Register Tool    ${fc_tool_payload}
    ${fc_tool_id}=    Get From Dictionary    ${fc_tool}    id

    ${fc_skill_tags}=    Create List    budget    management
    ${fc_skill_tool_deps}=    Create List    ${fc_tool_id}
    ${fc_skill_traits}=    Create List    ${fc_trait_id}
    ${fc_skill_def}=    Set Variable    \# Budget Management\\n\\nOversee departmental budgets, conduct variance analysis, and generate quarterly financial forecasts to guide executive decision-making.
    ${fc_skill_payload}=    Create Dictionary    name=BudgetManagement    owner_id=00000000-0000-0000-0000-000000000000    description=Adept at processing large volumes of financial data from ERPs to track spend against allocated budgets and identify cost-saving measures.    tags=${fc_skill_tags}    definition=${fc_skill_def}    tool_dependencies=${fc_skill_tool_deps}    implements_traits=${fc_skill_traits}
    ${fc_skill}=    Create Skill    ${fc_skill_payload}
    ${fc_skill_id}=    Get From Dictionary    ${fc_skill}    id

    ${fc_agent_tags}=    Create List    controller    finance
    ${fc_agent_traits}=    Create List    ${fc_trait_id}
    ${fc_agent_skills}=    Create List    ${fc_skill_id}
    ${fc_agent_tools}=    Create List    ${fc_tool_id}
    ${fc_agent_def}=    Set Variable    \# Financial Controller\\n\\nYou are a financial controller. Ensure the fiscal health of the organization by establishing policies, analyzing ERP data, and mitigating financial risks.
    ${fc_agent_payload}=    Create Dictionary    name=FinancialControllerAgent    owner_id=00000000-0000-0000-0000-000000000000    description=A high-level financial controller agent designed to manage corporate finance, enforce spending policies, and provide strategic fiscal insights.    agent_definition=${fc_agent_def}    tags=${fc_agent_tags}    implements_traits=${fc_agent_traits}    skill_dependencies=${fc_agent_skills}    tool_dependencies=${fc_agent_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${fc_agent}=    Create Agent    ${fc_agent_payload}

    # 9. Accounting and Financial Audit
    ${audit_trait_tags}=    Create List    accounting    audit    compliance
    ${audit_trait_payload}=    Create Dictionary    name=FinancialAudit    owner_id=00000000-0000-0000-0000-000000000000    description=Systematic examination of accounting records, financial statements, and internal controls to verify accuracy and ensure regulatory compliance.    capability_requirements=${empty_list}    behavioral_invariants=${empty_list}    evaluation_criteria=${empty_list}    tags=${audit_trait_tags}    guardrails=${empty_guardrails}
    ${audit_trait}=    Create Trait    ${audit_trait_payload}
    ${audit_trait_id}=    Get From Dictionary    ${audit_trait}    id

    ${audit_tool_tags}=    Create List    excel    csv    data
    ${audit_tool_config}=    Create Dictionary    url=http://localhost:3000/excel-sse    tags=${audit_tool_tags}    description=A utility for parsing and running computations on complex Excel and CSV financial spreadsheets
    ${audit_tool_schema}=    Create Dictionary    tools=${empty_list}    prompts=${empty_list}    resources=${empty_list}
    ${audit_tool_payload}=    Create Dictionary    server_name=excel-mcp-server    transport_type=sse    endpoint_config=${audit_tool_config}    schema=${audit_tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${audit_tool}=    Register Tool    ${audit_tool_payload}
    ${audit_tool_id}=    Get From Dictionary    ${audit_tool}    id

    ${audit_skill_tags}=    Create List    compliance    sox
    ${audit_skill_tool_deps}=    Create List    ${audit_tool_id}
    ${audit_skill_traits}=    Create List    ${audit_trait_id}
    ${audit_skill_def}=    Set Variable    \# Compliance Checking\\n\\nVerify organizational operations adhere to external laws and regulations, such as Sarbanes-Oxley (SOX), GAAP, and IFRS standards.
    ${audit_skill_payload}=    Create Dictionary    name=ComplianceChecking    owner_id=00000000-0000-0000-0000-000000000000    description=Meticulous skill for auditing ledgers, reconciling accounts, and detecting anomalies or potential fraud in accounting data.    tags=${audit_skill_tags}    definition=${audit_skill_def}    tool_dependencies=${audit_skill_tool_deps}    implements_traits=${audit_skill_traits}
    ${audit_skill}=    Create Skill    ${audit_skill_payload}
    ${audit_skill_id}=    Get From Dictionary    ${audit_skill}    id

    ${audit_agent_tags}=    Create List    auditor    accounting
    ${audit_agent_traits}=    Create List    ${audit_trait_id}
    ${audit_agent_skills}=    Create List    ${audit_skill_id}
    ${audit_agent_tools}=    Create List    ${audit_tool_id}
    ${audit_agent_def}=    Set Variable    \# Financial Auditor\\n\\nYou are a financial auditor. Inspect accounting books, perform independent audits, and assure stakeholders of the integrity of financial reporting.
    ${audit_agent_payload}=    Create Dictionary    name=FinancialAuditorAgent    owner_id=00000000-0000-0000-0000-000000000000    description=A detail-oriented accounting agent specialized in financial audits, general ledger reconciliation, and strict adherence to global financial regulations.    agent_definition=${audit_agent_def}    tags=${audit_agent_tags}    implements_traits=${audit_agent_traits}    skill_dependencies=${audit_agent_skills}    tool_dependencies=${audit_agent_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${audit_agent}=    Create Agent    ${audit_agent_payload}

    # 10. Quality Assurance
    ${qa_trait_tags}=    Create List    quality    assurance    testing
    ${qa_trait_payload}=    Create Dictionary    name=AutomatedTesting    owner_id=00000000-0000-0000-0000-000000000000    description=Design and implementation of automated test suites, including unit, integration, and end-to-end tests to ensure software quality and prevent regressions.    capability_requirements=${empty_list}    behavioral_invariants=${empty_list}    evaluation_criteria=${empty_list}    tags=${qa_trait_tags}    guardrails=${empty_guardrails}
    ${qa_trait}=    Create Trait    ${qa_trait_payload}
    ${qa_trait_id}=    Get From Dictionary    ${qa_trait}    id

    ${qa_tool_tags}=    Create List    selenium    browser    automation
    ${qa_tool_config}=    Create Dictionary    url=http://localhost:3000/selenium-sse    tags=${qa_tool_tags}    description=Browser automation tool for executing UI tests via Selenium WebDriver
    ${qa_tool_schema}=    Create Dictionary    tools=${empty_list}    prompts=${empty_list}    resources=${empty_list}
    ${qa_tool_payload}=    Create Dictionary    server_name=selenium-mcp-server    transport_type=sse    endpoint_config=${qa_tool_config}    schema=${qa_tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${qa_tool}=    Register Tool    ${qa_tool_payload}
    ${qa_tool_id}=    Get From Dictionary    ${qa_tool}    id

    ${qa_skill_tags}=    Create List    e2e    testing
    ${qa_skill_tool_deps}=    Create List    ${qa_tool_id}
    ${qa_skill_traits}=    Create List    ${qa_trait_id}
    ${qa_skill_def}=    Set Variable    \# End To End Testing\\n\\nWrite and execute comprehensive end-to-end test scenarios simulating real user journeys across the application stack.
    ${qa_skill_payload}=    Create Dictionary    name=EndToEndTesting    owner_id=00000000-0000-0000-0000-000000000000    description=Skill dedicated to validating integrated software components and user interfaces against predefined requirements using browser automation.    tags=${qa_skill_tags}    definition=${qa_skill_def}    tool_dependencies=${qa_skill_tool_deps}    implements_traits=${qa_skill_traits}
    ${qa_skill}=    Create Skill    ${qa_skill_payload}
    ${qa_skill_id}=    Get From Dictionary    ${qa_skill}    id

    ${qa_agent_tags}=    Create List    qa    engineer
    ${qa_agent_traits}=    Create List    ${qa_trait_id}
    ${qa_agent_skills}=    Create List    ${qa_skill_id}
    ${qa_agent_tools}=    Create List    ${qa_tool_id}
    ${qa_agent_def}=    Set Variable    \# QA Automation Engineer\\n\\nYou are a QA automation engineer. Your goal is to break the software through rigorous automated testing and ensure all bug fixes are accompanied by regression tests.
    ${qa_agent_payload}=    Create Dictionary    name=QAAutomationEngineerAgent    owner_id=00000000-0000-0000-0000-000000000000    description=An agent specializing in software quality assurance, capable of writing resilient automated tests and identifying edge cases.    agent_definition=${qa_agent_def}    tags=${qa_agent_tags}    implements_traits=${qa_agent_traits}    skill_dependencies=${qa_agent_skills}    tool_dependencies=${qa_agent_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${qa_agent}=    Create Agent    ${qa_agent_payload}

    # 11. Data Science
    ${ds_trait_tags}=    Create List    data    science    analysis
    ${ds_trait_payload}=    Create Dictionary    name=DataAnalysis    owner_id=00000000-0000-0000-0000-000000000000    description=Extraction of actionable insights from complex, structured, and unstructured datasets using statistical methods and exploratory data analysis.    capability_requirements=${empty_list}    behavioral_invariants=${empty_list}    evaluation_criteria=${empty_list}    tags=${ds_trait_tags}    guardrails=${empty_guardrails}
    ${ds_trait}=    Create Trait    ${ds_trait_payload}
    ${ds_trait_id}=    Get From Dictionary    ${ds_trait}    id

    ${ds_tool_tags}=    Create List    jupyter    python    notebook
    ${ds_tool_config}=    Create Dictionary    url=http://localhost:3000/jupyter-sse    tags=${ds_tool_tags}    description=Integration with Jupyter notebooks for executing Python data science workloads
    ${ds_tool_schema}=    Create Dictionary    tools=${empty_list}    prompts=${empty_list}    resources=${empty_list}
    ${ds_tool_payload}=    Create Dictionary    server_name=jupyter-mcp-server    transport_type=sse    endpoint_config=${ds_tool_config}    schema=${ds_tool_schema}    owner_id=00000000-0000-0000-0000-000000000001
    ${ds_tool}=    Register Tool    ${ds_tool_payload}
    ${ds_tool_id}=    Get From Dictionary    ${ds_tool}    id

    ${ds_skill_tags}=    Create List    machine    learning    ml
    ${ds_skill_tool_deps}=    Create List    ${ds_tool_id}
    ${ds_skill_traits}=    Create List    ${ds_trait_id}
    ${ds_skill_def}=    Set Variable    \# Machine Learning Modeling\\n\\nTrain, evaluate, and deploy predictive models using libraries like scikit-learn, TensorFlow, or PyTorch on large datasets.
    ${ds_skill_payload}=    Create Dictionary    name=MachineLearningModeling    owner_id=00000000-0000-0000-0000-000000000000    description=Advanced analytical skill for building algorithms that learn from and make predictions on data.    tags=${ds_skill_tags}    definition=${ds_skill_def}    tool_dependencies=${ds_skill_tool_deps}    implements_traits=${ds_skill_traits}
    ${ds_skill}=    Create Skill    ${ds_skill_payload}
    ${ds_skill_id}=    Get From Dictionary    ${ds_skill}    id

    ${ds_agent_tags}=    Create List    data    scientist
    ${ds_agent_traits}=    Create List    ${ds_trait_id}
    ${ds_agent_skills}=    Create List    ${ds_skill_id}
    ${ds_agent_tools}=    Create List    ${ds_tool_id}
    ${ds_agent_def}=    Set Variable    \# Data Scientist\\n\\nYou are a data scientist. Clean messy datasets, perform exploratory analysis, and build robust machine learning models to solve business problems.
    ${ds_agent_payload}=    Create Dictionary    name=DataScientistAgent    owner_id=00000000-0000-0000-0000-000000000000    description=A specialized agent for handling data science workflows, from data wrangling to model deployment and visualization.    agent_definition=${ds_agent_def}    tags=${ds_agent_tags}    implements_traits=${ds_agent_traits}    skill_dependencies=${ds_agent_skills}    tool_dependencies=${ds_agent_tools}    model=claude-3-5-sonnet-v2    guardrails=${empty_guardrails}
    ${ds_agent}=    Create Agent    ${ds_agent_payload}

    Log    Successfully seeded exemplar data!
