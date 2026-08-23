import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { ApiService } from '../../services/api.service';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { NO_ERRORS_SCHEMA } from '@angular/core';
import { provideRouter } from '@angular/router';
import { AgentRegistryComponent } from './agent-registry.component';

describe('AgentRegistryComponent', () => {
  let component: AgentRegistryComponent;
  let fixture: ComponentFixture<AgentRegistryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentRegistryComponent],
      providers: [provideHttpClient(), provideAnimationsAsync(), provideRouter([]), { provide: ActivatedRoute, useValue: { queryParams: of({}), snapshot: { paramMap: { get: () => null } } } }, { provide: ApiService, useValue: { getAgents: () => of([]), getTraits: () => of({ids: []}), getTrait: () => of(null), getMcpServers: () => of([]), getSkills: () => of([]) } }],
      schemas: [NO_ERRORS_SCHEMA]
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
        version: '1',
        owner_id: 'test_owner',
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
