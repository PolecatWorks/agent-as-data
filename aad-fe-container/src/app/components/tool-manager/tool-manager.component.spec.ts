import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { ToolManagerComponent } from './tool-manager.component';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { NO_ERRORS_SCHEMA } from '@angular/core';
import { ApiService } from '../../services/api.service';

describe('ToolManagerComponent', () => {
  let component: ToolManagerComponent;
  let fixture: ComponentFixture<ToolManagerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ToolManagerComponent],
      providers: [provideHttpClient(), provideAnimationsAsync(), { provide: ActivatedRoute, useValue: { paramMap: of({ get: () => null }), queryParams: of({}), snapshot: { paramMap: { get: () => null } } } }],
      schemas: [NO_ERRORS_SCHEMA]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ToolManagerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render the workspace title view switcher dropdown trigger', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const titleSwitcher = compiled.querySelector('[data-testid="workspace-title-switcher"]');
    expect(titleSwitcher).toBeTruthy();
    expect(titleSwitcher?.textContent).toContain('Tools Registry');
  });

  it('should render the zero-footprint concept guide for tools', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const trigger = compiled.querySelector('[data-testid="tools-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger?.textContent).toContain('What are Tools?');
  });

  it('should default transport_type to http when starting a new server', () => {
    component.startNewServer();
    expect(component.serverForm.transport_type).toBe('http');
    expect(component.serverForm.url).toBe('http://localhost:8082');
    expect(component.isEditing).toBeTrue();
  });

  it('should call apiService.syncTool when syncServer is invoked', () => {
    const apiService = TestBed.inject(ApiService);
    spyOn(apiService, 'syncTool').and.returnValue(of({
      id: 'tool-1',
      server_name: 'test-mcp',
      cached_tools_count: 3,
      sync_status: 'synced',
      last_synced_at: new Date().toISOString(),
      last_sync_error: null
    }));
    spyOn(apiService, 'getTools').and.returnValue(of([]));

    component.selectedServer = {
      id: 'tool-1',
      server_name: 'test-mcp',
      tools_count: 1,
      sync_status: 'degraded'
    };

    component.syncServer();
    expect(apiService.syncTool).toHaveBeenCalledWith('tool-1');
    expect(component.selectedServer.tools_count).toBe(3);
    expect(component.selectedServer.sync_status).toBe('synced');
  });

  it('should open tool tester and auto-populate arguments from schema', () => {
    const mockTool = {
      name: 'hello',
      description: 'Greet user',
      inputSchema: {
        type: 'object',
        properties: {
          name: { type: 'string', description: 'User name' }
        },
        required: ['name']
      }
    };

    component.openToolTester(mockTool);
    expect(component.activeTestTool).toBe(mockTool);
    expect(component.testArgsJson).toContain('"name": "Antigravity"');

    const props = component.getToolProperties(mockTool);
    expect(props.length).toBe(1);
    expect(props[0].name).toBe('name');
    expect(props[0].required).toBeTrue();
  });

  it('should call apiService.testTool when executeToolTest is invoked', () => {
    const apiService = TestBed.inject(ApiService);
    spyOn(apiService, 'testTool').and.returnValue(of({
      success: true,
      tool_name: 'hello',
      output: 'Hello, Antigravity!',
      raw_result: { content: [{ type: 'text', text: 'Hello, Antigravity!' }] },
      latency_ms: 12
    }));

    component.selectedServer = { id: 'srv-1', server_name: 'test-srv' };
    component.activeTestTool = { name: 'hello' };
    component.testArgsJson = '{"name": "Antigravity"}';

    component.executeToolTest();
    expect(apiService.testTool).toHaveBeenCalledWith('srv-1', 'hello', { name: 'Antigravity' });
    expect(component.testResult).toBeTruthy();
    expect(component.testResult.success).toBeTrue();
    expect(component.testResult.output).toBe('Hello, Antigravity!');
  });
});
