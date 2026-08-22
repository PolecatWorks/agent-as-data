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
        provideRouter([])
      ]
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
