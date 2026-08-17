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

  it('should retrieve trait contract descriptions on hover/query', () => {
    component.traitContracts = [
      {
        id: '1',
        name: 'SecurityAuditor',
        description: 'Vulnerability scanner.',
        version: 1,
        capability_requirements: [],
        behavioral_invariants: [],
        evaluation_criteria: [],
        tags: []
      }
    ];

    expect(component.getTraitDescription('SecurityAuditor')).toBe('Vulnerability scanner.');
    expect(component.getTraitDescription('NonExistent')).toBe('No description available');
  });
});
