import { ComponentFixture, TestBed } from '@angular/core/testing';

import { CodeEditorTextareaComponent } from './code-editor-textarea.component';

describe('CodeEditorTextareaComponent', () => {
  let component: CodeEditorTextareaComponent;
  let fixture: ComponentFixture<CodeEditorTextareaComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [CodeEditorTextareaComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(CodeEditorTextareaComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
