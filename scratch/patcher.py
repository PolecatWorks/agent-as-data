import re

with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.ts', 'r') as f:
    code = f.read()

# 1. Replace ngOnInit
ngOnInit_replacement = """
  ngOnInit(): void {
    // We still load lists to populate the Attachment Manager dropdowns
    this.loadAgents(); // loads all agents so we can select sub-agents
    this.loadTraits();
    this.loadTools();
    this.loadSkills();
    
    // Listen to route params to fetch the specific agent
    this.route.paramMap.subscribe(params => {
      const id = params.get('id');
      if (this.route.snapshot.routeConfig?.path === "new") {
        this.selectedAgent = null;
        this.isEditing = true;
        this.agentForm = {
          name: '',
          description: '',
          tags: [],
          implements_traits: [],
          uses_traits: [],
          attached_tools: [],
          attached_agents: [],
          attached_skills: [],
          current_version: '1.0.0',
          judge_threshold: 0.8,
          model: 'claude-3-5-sonnet-v2',
          agent_definition: '',
        };
      } else if (id) {
        this.loadAgent(id);
      }
    });

    this.route.queryParams.subscribe((queryParams) => {
      this.isEditing = queryParams['edit'] === 'true';
    });
  }
"""
code = re.sub(r'  ngOnInit\(\): void \{[\s\S]*?\}\n\n', ngOnInit_replacement[1:] + "\n", code, count=1)


# 2. Replace loadAgents and applySelectedAgentFromRoute with loadAgent and loadAgents
loadAgents_replacement = """
  loadAgent(id: string): void {
    this.apiService.getAgent(id).subscribe({
      next: (fullAgent) => {
        this.selectedAgent = fullAgent;
        this.agentForm = {
          ...fullAgent,
          guardrails: fullAgent.guardrails || {
            input_guardrails: {
              active_guardrails: fullAgent.input_guardrails?.map((gType: string) => ({
                id: 'g-' + Math.random().toString(36).substring(2, 9),
                type: gType,
                name: gType.replace('_', ' ').toUpperCase(),
                tier: 'Deterministic',
                description: 'Imported guardrail constraint',
                config: {},
              })) || [],
            },
            output_guardrails: {
              active_guardrails: fullAgent.output_guardrails?.map((gType: string) => ({
                id: 'og-' + Math.random().toString(36).substring(2, 9),
                type: gType,
                name: gType.replace('_', ' ').toUpperCase(),
                tier: 'Deterministic',
                description: 'Imported guardrail constraint',
                config: {},
              })) || [],
            },
          },
        };
        // If edit mode query param is set, keep editing
        if (!this.isEditing) {
            this.isEditing = false;
        }
      },
      error: (err) => {
        this.snackBar.open(`Error loading agent details`, 'Close', { duration: 3000 });
      }
    });
  }

  loadAgents(): void {
    this.apiService.getAgents().subscribe({
      next: (data) => {
        this.agents = data;
      }
    });
  }
"""
code = re.sub(r'  loadAgents\(\): void \{[\s\S]*?  private applySelectedAgentFromRoute\(routeId: string \| null\): void \{[\s\S]*?  \}\n\n', loadAgents_replacement[1:] + "\n", code, count=1)


# 3. Remove selectAgent
code = re.sub(r'  selectAgent\(agent: Agent, keepEdit = false\): void \{[\s\S]*?    \}\n  \}\n\n', "", code, count=1)


# 4. Replace cancelEdit
cancelEdit_replacement = """
  cancelEdit(): void {
    this.isEditing = false;
    if (this.selectedAgent && this.selectedAgent.id) {
      this.loadAgent(this.selectedAgent.id);
      this.router.navigate(["/agents", this.selectedAgent.id]);
    } else {
      this.router.navigate(["/agents"]);
    }
  }
"""
code = re.sub(r'  cancelEdit\(\): void \{[\s\S]*?  \}\n\n', cancelEdit_replacement[1:] + "\n", code, count=1)


# 5. Remove createNewAgent (it's handled by Layout/Empty Component now)
code = re.sub(r'  createNewAgent\(\): void \{[\s\S]*?    \};\n  \}\n\n', "", code, count=1)

# 6. Replace saveAgent
saveAgent_replacement = """
  saveAgent(): void {
    const payload = this.preparePayload();

    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.updateAgent(this.selectedAgent.id, payload).subscribe({
        next: (res) => {
          this.snackBar.open('Agent updated successfully!', 'Close', { duration: 3000 });
          this.loadAgent(this.selectedAgent!.id!);
        },
        error: () => {
          this.snackBar.open('Updated agent specifications locally.', 'Close', { duration: 3000 });
        },
      });
    } else {
      this.apiService.createAgent(payload).subscribe({
        next: (newAgent) => {
          this.snackBar.open('Agent created successfully!', 'Close', { duration: 3000 });
          const newId = newAgent.id || (newAgent as any).agent_id;
          this.router.navigate(['/agents', newId]);
        },
        error: () => {
          this.snackBar.open('Created new agent locally.', 'Close', { duration: 3000 });
        },
      });
    }
  }
"""
code = re.sub(r'  saveAgent\(\): void \{[\s\S]*?    \}\n  \}\n\n', saveAgent_replacement[1:] + "\n", code, count=1)

# 7. Replace deleteAgent
deleteAgent_replacement = """
  deleteAgent(): void {
    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.deleteAgent(this.selectedAgent.id).subscribe({
        next: (deletedAgent) => {
          this.snackBar.open('Deleted agent successfully!', 'Close', { duration: 3000 });
          this.router.navigate(['/agents']);
        },
        error: () => {
          this.snackBar.open('Deleted agent specifications locally.', 'Close', { duration: 3000 });
          this.router.navigate(['/agents']);
        },
      });
    }
  }
"""
code = re.sub(r'  deleteAgent\(\): void \{[\s\S]*?    \}\n  \}\n\n', deleteAgent_replacement[1:] + "\n", code, count=1)

with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.ts', 'w') as f:
    f.write(code)

