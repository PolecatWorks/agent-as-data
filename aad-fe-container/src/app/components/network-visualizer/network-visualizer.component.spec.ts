import { ComponentFixture, TestBed } from '@angular/core/testing';

import { NetworkVisualizerComponent } from './network-visualizer.component';

describe('NetworkVisualizerComponent', () => {
  let component: NetworkVisualizerComponent;
  let fixture: ComponentFixture<NetworkVisualizerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [NetworkVisualizerComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(NetworkVisualizerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
