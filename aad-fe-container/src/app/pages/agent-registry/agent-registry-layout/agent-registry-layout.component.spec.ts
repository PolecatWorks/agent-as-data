import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AgentRegistryLayoutComponent } from './agent-registry-layout.component';
import { HttpClientTestingModule } from '@angular/common/http/testing';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';

describe('AgentRegistryLayoutComponent', () => {
  let component: AgentRegistryLayoutComponent;
  let fixture: ComponentFixture<AgentRegistryLayoutComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentRegistryLayoutComponent, HttpClientTestingModule],
      providers: [
        { provide: ActivatedRoute, useValue: { params: of({}) } }
      ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(AgentRegistryLayoutComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
