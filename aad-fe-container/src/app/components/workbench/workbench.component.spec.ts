import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HttpClientTestingModule } from '@angular/common/http/testing';
import { RouterModule } from '@angular/router';
import { of } from 'rxjs';

import { WorkbenchComponent } from './workbench.component';
import { ApiService, ThreadRun, Thread } from '../../services/api.service';

describe('WorkbenchComponent', () => {
  let component: WorkbenchComponent;
  let fixture: ComponentFixture<WorkbenchComponent>;
  let apiService: ApiService;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WorkbenchComponent, HttpClientTestingModule, RouterModule.forRoot([])]
    })
    .compileComponents();

    fixture = TestBed.createComponent(WorkbenchComponent);
    component = fixture.componentInstance;
    apiService = TestBed.inject(ApiService);
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should detect an active running task on thread load and enter isProcessing state', () => {
    const mockRun: ThreadRun = {
      id: 'run-123',
      thread_id: 'thread-1',
      bench_id: 'bench-1',
      status: 'running',
      current_phase: 'thinking',
      active_tool_name: null
    };
    spyOn(apiService, 'getActiveThreadRun').and.returnValue(of(mockRun));

    component.checkActiveRun('thread-1');

    expect(component.isProcessing).toBeTrue();
    expect(component.activeRun).toEqual(mockRun);
  });

  it('should invoke cancelActiveThreadRun and reset isProcessing when cancelCurrentAction is clicked', () => {
    const mockThread: Thread = {
      id: 'thread-1',
      bench_id: 'bench-1',
      owner_id: 'owner-1',
      title: 'Test Thread',
      created_at: new Date().toISOString()
    };
    component.activeThread = mockThread;
    component.isProcessing = true;
    component.activeRun = {
      id: 'run-123',
      thread_id: 'thread-1',
      bench_id: 'bench-1',
      status: 'running',
      current_phase: 'thinking'
    };

    spyOn(apiService, 'cancelActiveThreadRun').and.returnValue(of({ message: 'Run cancelled', status: 'cancelled' }));
    spyOn(apiService, 'getMessages').and.returnValue(of([
      { id: 'm1', thread_id: 'thread-1', role: 'user', content: 'Hello', created_at: '' },
      { id: 'm2', thread_id: 'thread-1', role: 'system', content: '[Action cancelled by user]', created_at: '' }
    ]));

    component.cancelCurrentAction();

    expect(apiService.cancelActiveThreadRun).toHaveBeenCalledWith('thread-1');
    expect(component.isProcessing).toBeFalse();
    expect(component.activeRun).toBeNull();
    expect(component.activeThreadMessages.length).toBe(2);
    expect(component.activeThreadMessages[1].role).toBe('system');
  });

  it('should render the zero-footprint concept guide trigger and configuration in top bar', () => {
    const trigger = fixture.nativeElement.querySelector('[data-testid="workbench-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What are Workbenches?');

    expect(component.conceptGuideMappings.length).toBe(3);
    expect(component.conceptGuideMappings[0].title).toBe('1. Sandboxed Filesystem');
    expect(component.conceptGuideMappings[1].title).toBe('2. Conversational Threads');
    expect(component.conceptGuideMappings[2].title).toBe('3. Shared Bench Memory');
  });

  it('should render the workspace title as an interactive view switcher trigger with dropdown affordance', () => {
    const switcher = fixture.nativeElement.querySelector('[data-testid="workspace-title-switcher"]');
    expect(switcher).toBeTruthy();
    expect(switcher.textContent).toContain('Workbench');
    expect(switcher.textContent).toContain('expand_more');
  });

  describe('Tool Execution Cards', () => {
    it('should parse historical JSON codeblock tool execution message with success: true', () => {
      const raw = 'Executed `write_file`:\n```json\n{"success":true,"message":"Successfully wrote to ben.md"}\n```';
      const parsed = component.parseToolExecution(raw);
      expect(parsed).not.toBeNull();
      expect(parsed?.toolName).toBe('write_file');
      expect(parsed?.success).toBeTrue();
      expect(parsed?.message).toBe('Successfully wrote to ben.md');
    });

    it('should parse historical JSON codeblock tool execution message with success: false', () => {
      const raw = 'Executed `delete_file`:\n```json\n{"success":false,"message":"File not found"}\n```';
      const parsed = component.parseToolExecution(raw);
      expect(parsed).not.toBeNull();
      expect(parsed?.toolName).toBe('delete_file');
      expect(parsed?.success).toBeFalse();
      expect(parsed?.message).toBe('File not found');
    });

    it('should parse structured single-line tool execution format', () => {
      const raw = 'Executed `replace_in_file` (success): Replaced 1 occurrence';
      const parsed = component.parseToolExecution(raw);
      expect(parsed).not.toBeNull();
      expect(parsed?.toolName).toBe('replace_in_file');
      expect(parsed?.success).toBeTrue();
      expect(parsed?.message).toBe('Replaced 1 occurrence');
    });

    it('should return null for standard user and assistant messages', () => {
      expect(component.parseToolExecution('Hello world')).toBeNull();
      expect(component.parseToolExecution('I have created the file for you.')).toBeNull();
      expect(component.parseToolExecution('')).toBeNull();
    });

    it('should render Tool Execution Card in template instead of raw JSON dump', () => {
      component.activeThread = {
        id: 'thread-1',
        bench_id: 'bench-1',
        owner_id: 'owner-1',
        title: 'Test Thread',
        created_at: new Date().toISOString()
      };
      component.activeThreadMessages = [
        {
          id: 'msg-1',
          thread_id: 'thread-1',
          role: 'assistant',
          content: 'Executed `write_file`:\n```json\n{"success":true,"message":"Successfully wrote to ben.md"}\n```',
          created_at: new Date().toISOString()
        }
      ];
      fixture.detectChanges();

      const card = fixture.nativeElement.querySelector('[data-testid="tool-execution-card"]');
      expect(card).toBeTruthy();
      expect(card.textContent).toContain('write_file');
      expect(card.textContent).toContain('Success');
      expect(card.textContent).toContain('Successfully wrote to ben.md');
      expect(card.textContent).not.toContain('```json');
    });
  });
});



