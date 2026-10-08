import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { NetworkVisualizerComponent } from './network-visualizer.component';

describe('NetworkVisualizerComponent', () => {
  let component: NetworkVisualizerComponent;
  let fixture: ComponentFixture<NetworkVisualizerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [NetworkVisualizerComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([]),
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(NetworkVisualizerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render the workspace title ', () => {
    const switcher = fixture.nativeElement.querySelector(
      '[data-testid="workspace-title"]',
    );
    expect(switcher).toBeTruthy();
    expect(switcher.textContent).toContain('Network Graph Visualizer');
  });

  it('should render the zero-footprint concept guide trigger and configuration in top bar', () => {
    const trigger = fixture.nativeElement.querySelector(
      '[data-testid="network-concept-trigger"]',
    );
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What is Network Graph?');

    expect(component.conceptGuideMappings.length).toBe(3);
    expect(component.conceptGuideMappings[0].title).toBe(
      '1. Teammate Node Hierarchies',
    );
    expect(component.conceptGuideMappings[1].title).toBe(
      '2. Trait Contract Boundaries',
    );
    expect(component.conceptGuideMappings[2].title).toBe(
      '3. Skill Delegation Edges',
    );
  });
});
