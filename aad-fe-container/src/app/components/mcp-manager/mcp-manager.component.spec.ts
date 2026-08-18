import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { McpManagerComponent } from './mcp-manager.component';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { NO_ERRORS_SCHEMA } from '@angular/core';

describe('McpManagerComponent', () => {
  let component: McpManagerComponent;
  let fixture: ComponentFixture<McpManagerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [McpManagerComponent],
      providers: [provideHttpClient(), provideAnimationsAsync(), { provide: ActivatedRoute, useValue: { paramMap: of({ get: () => null }), queryParams: of({}), snapshot: { paramMap: { get: () => null } } } }],
      schemas: [NO_ERRORS_SCHEMA]
    })
    .compileComponents();

    fixture = TestBed.createComponent(McpManagerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
