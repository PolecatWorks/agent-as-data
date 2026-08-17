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
});
