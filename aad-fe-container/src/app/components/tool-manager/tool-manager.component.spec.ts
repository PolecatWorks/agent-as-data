import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { ToolManagerComponent } from './tool-manager.component';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { NO_ERRORS_SCHEMA } from '@angular/core';

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
});
