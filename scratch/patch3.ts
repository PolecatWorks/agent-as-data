    this.route.paramMap.subscribe(params => {
      const id = params.get('id');
      if (id === 'new') {
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
