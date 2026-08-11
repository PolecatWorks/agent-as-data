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
});
