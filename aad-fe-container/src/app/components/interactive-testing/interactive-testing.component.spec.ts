import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { of } from 'rxjs';
import { InteractiveTestingComponent, TestEntity } from './interactive-testing.component';
import { ApiService, Agent, Skill } from '../../services/api.service';

describe('InteractiveTestingComponent', () => {
  let component: InteractiveTestingComponent;
  let fixture: ComponentFixture<InteractiveTestingComponent>;
  let apiService: ApiService;

  const mockAgents: Agent[] = [
    {
      id: 'agent-1111',
      name: 'Security Auditor Agent',
      description: 'Reviews code for vulnerability flaws and injection vectors.',
      agent_definition: 'You are a paranoid security auditor. Scan for OWASP top 10.',
      tags: ['security', 'auditor'],
      current_version: '1.2.0',
      owner_id: '00000000-0000-0000-0000-000000000000',
      judge_threshold: 0.85,
      implements_traits: ['SecurityAuditor'],
      uses_traits: [],
      model: 'qwen2.5-coder:14b',
      attached_skills: ['skill-2222'],
      attached_tools: ['tool-3333']
    }
  ];

  const mockSkills: Skill[] = [
    {
      id: 'skill-2222',
      name: 'JSON Log Formatter',
      description: 'Formats raw execution logs into structured JSON objects.',
      definition: 'Format all incoming text into { timestamp, level, message } JSON.',
      tags: ['logging', 'json'],
      current_version: '1.0.0',
      owner_id: '00000000-0000-0000-0000-000000000000'
    }
  ];

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [InteractiveTestingComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([])
      ]
    })
    .compileComponents();

    apiService = TestBed.inject(ApiService);
    spyOn(apiService, 'getAgents').and.returnValue(of(mockAgents));
    spyOn(apiService, 'getSkills').and.returnValue(of(mockSkills));

    fixture = TestBed.createComponent(InteractiveTestingComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create and load initial entities', () => {
    expect(component).toBeTruthy();
    expect(component.allEntities.length).toBe(2);
    expect(component.selectedEntity).toBeTruthy();
    expect(component.selectedEntity?.name).toBe('Security Auditor Agent');
    expect(component.selectedEntity?.definition).toBe('You are a paranoid security auditor. Scan for OWASP top 10.');
    expect(component.selectedModel).toBe('qwen2.5-coder:14b');
  });

  it('should toggle sidebar collapsed state', () => {
    expect(component.isSidebarCollapsed).toBeFalse();
    component.toggleSidebar();
    expect(component.isSidebarCollapsed).toBeTrue();
    component.toggleSidebar();
    expect(component.isSidebarCollapsed).toBeFalse();
  });

  it('should toggle prompt inspector expanded state', () => {
    expect(component.isPromptExpanded).toBeTrue();
    component.togglePromptExpanded();
    expect(component.isPromptExpanded).toBeFalse();
  });

  it('should filter entities by query', () => {
    component.searchQuery = 'JSON';
    const filtered = component.getFilteredEntities();
    expect(filtered.length).toBe(1);
    expect(filtered[0].name).toBe('JSON Log Formatter');
  });

  it('should select a skill entity and update context', () => {
    const skillEntity: TestEntity = component.allEntities.find(e => e.type === 'skill')!;
    component.selectEntity(skillEntity);
    expect(component.selectedEntity?.id).toBe('skill-2222');
    expect(component.selectedEntity?.definition).toBe('Format all incoming text into { timestamp, level, message } JSON.');
  });

  it('should allow changing Ollama model', () => {
    component.selectModel('llama3.2:3b');
    expect(component.selectedModel).toBe('llama3.2:3b');
  });

  it('should execute prompt via Rig / Ollama and display final output', () => {
    spyOn(apiService, 'executeAgent').and.returnValue(of({
      execution_id: 'exec-9999',
      agent_id: 'agent-1111',
      status: 'completed',
      output: 'No critical memory safety vulnerabilities detected.',
      execution_version: 1
    }));

    component.promptInput = 'Check for buffer overflows.';
    component.runExecution();

    expect(apiService.executeAgent).toHaveBeenCalledWith(
      'agent-1111',
      'Check for buffer overflows.',
      undefined,
      'qwen2.5-coder:14b'
    );
    expect(component.finalOutput).toBe('No critical memory safety vulnerabilities detected.');
    expect(component.executionOutput).toContain('No critical memory safety vulnerabilities detected.');
    expect(component.isExecuting).toBeFalse();
  });
});
