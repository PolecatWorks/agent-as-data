import { ComponentFixture, TestBed } from '@angular/core/testing';

import { KnowledgeInspectorComponent } from './knowledge-inspector.component';

describe('KnowledgeInspectorComponent', () => {
  let component: KnowledgeInspectorComponent;
  let fixture: ComponentFixture<KnowledgeInspectorComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [KnowledgeInspectorComponent]
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
