import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { AgentRegistryComponent } from './agent-registry.component';

describe('AgentRegistryComponent', () => {
  let component: AgentRegistryComponent;
  let fixture: ComponentFixture<AgentRegistryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentRegistryComponent],
      providers: [provideHttpClient(), provideAnimationsAsync(), provideRouter([])]
    })
    .compileComponents();


    fixture = TestBed.createComponent(AgentRegistryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should not add duplicate metadata tags', () => {
    component.agentForm = {
      tags: ['security', 'audit']
    };
    component.newTag = 'security';
    component.addTag();
    expect(component.agentForm.tags).toEqual(['security', 'audit']);
    
    component.newTag = 'new-tag';
    component.addTag();
    expect(component.agentForm.tags).toEqual(['security', 'audit', 'new-tag']);
  });

  it('should only advance trait contract version when capability requirements, behavioral invariants, or evaluation criteria change', () => {
    component.traitContracts = [
      {
        id: 'test-trait',
        name: 'TestTrait',
        description: 'Original description',
        version: 1,
        capability_requirements: ['Req1'],
        behavioral_invariants: ['Inv1'],
        evaluation_criteria: ['Crit1'],
        tags: ['tag1']
      }
    ];

    // Case 1: Changing only description and tags -> version should remain 1
    component.selectedTraitContract = component.traitContracts[0];
    component.traitForm = {
      name: 'TestTrait',
      description: 'Modified description',
      version: 1,
      capability_requirements: ['Req1'],
      behavioral_invariants: ['Inv1'],
      evaluation_criteria: ['Crit1'],
      tags: ['tag1', 'tag2']
    };

    component.saveTraitContract();
    expect(component.traitContracts[0].version).toBe(1);
    expect(component.traitContracts[0].description).toBe('Modified description');
    expect(component.traitContracts[0].tags).toEqual(['tag1', 'tag2']);

    // Case 2: Changing capability requirements -> version should advance to 2
    component.traitForm.capability_requirements = ['Req1', 'Req2'];
    component.saveTraitContract();
    expect(component.traitContracts[0].version).toBe(2);

    // Case 3: Changing behavioral invariants -> version should advance to 3
    component.traitForm.behavioral_invariants = ['Inv1', 'Inv2'];
    component.saveTraitContract();
    expect(component.traitContracts[0].version).toBe(3);

    // Case 4: Changing evaluation criteria -> version should advance to 4
    component.traitForm.evaluation_criteria = ['Crit1', 'Crit2'];
    component.saveTraitContract();
    expect(component.traitContracts[0].version).toBe(4);
  });

  it('should only advance agent version when functional fields change, and not on description or tags change', () => {
    const originalAgent: any = {
      id: 'agent-123',
      name: 'AgentOne',
      description: 'Original description',
      tags: ['tag1'],
      implements_traits: ['Trait1'],
      attached_tools: ['tool1'],
      attached_skills: ['skill1'],
      attached_agents: ['sub-agent1'],
      current_version: 1,
      owner_id: 'owner-123',
      judge_threshold: 0.8,
      model: 'gpt-4o',
      agent_definition: 'Original prompt',
      guardrails: {
        input_guardrails: { active_guardrails: [{ id: '1', type: 'prompt_injection', name: 'PI', tier: 'Deterministic', description: 'desc', config: {} }] },
        output_guardrails: { active_guardrails: [] }
      }
    };

    component.selectedAgent = originalAgent;
    component.agentForm = {
      ...originalAgent,
      guardrails: {
        input_guardrails: { active_guardrails: [{ id: '1', type: 'prompt_injection', name: 'PI', tier: 'Deterministic', description: 'desc', config: {} }] },
        output_guardrails: { active_guardrails: [] }
      }
    };

    // Case 1: Changing description and tags -> version remains 1
    component.agentForm.description = 'Updated description';
    component.agentForm.tags = ['tag1', 'tag2'];
    
    let payload = (component as any).preparePayload();
    expect(payload.current_version).toBe(1);

    // Case 2: Changing name -> version advances to 2
    component.agentForm.name = 'AgentTwo';
    payload = (component as any).preparePayload();
    expect(payload.current_version).toBe(2);

    // Reset name and Case 3: Changing agent_definition -> version advances to 2
    component.agentForm.name = 'AgentOne';
    component.agentForm.agent_definition = 'New prompt';
    payload = (component as any).preparePayload();
    expect(payload.current_version).toBe(2);

    // Reset agent_definition and Case 4: Changing tools -> version advances to 2
    component.agentForm.agent_definition = 'Original prompt';
    component.agentForm.attached_tools = ['tool1', 'tool2'];
    payload = (component as any).preparePayload();
    expect(payload.current_version).toBe(2);
  });
});
