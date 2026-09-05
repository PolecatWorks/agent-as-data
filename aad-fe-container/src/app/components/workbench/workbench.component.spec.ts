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
});

