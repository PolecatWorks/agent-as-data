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
    // Only used for the 'available agents to attach' dropdown
    this.apiService.getAgents().subscribe({
      next: (data) => {
        this.agents = data;
      }
    });
  }
