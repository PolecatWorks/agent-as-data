import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { KnowledgeInspectorComponent } from './knowledge-inspector.component';

describe('KnowledgeInspectorComponent', () => {
  let component: KnowledgeInspectorComponent;
  let fixture: ComponentFixture<KnowledgeInspectorComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [KnowledgeInspectorComponent],
      providers: [provideHttpClient(), provideAnimationsAsync()]
    })
    .compileComponents();

    fixture = TestBed.createComponent(KnowledgeInspectorComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
