import { Component, forwardRef, Input } from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR, FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-code-editor-textarea',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './code-editor-textarea.component.html',
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => CodeEditorTextareaComponent),
      multi: true
    }
  ]
})
export class CodeEditorTextareaComponent implements ControlValueAccessor {
  @Input() placeholder: string = '';
  @Input() rows: number = 10;
  @Input() codeColor: 'emerald' | 'indigo' | 'amber' | 'slate' = 'emerald';
  @Input() disabled: boolean = false;

  value: string = '';

  onChange: any = () => {};
  onTouch: any = () => {};

  writeValue(value: any): void {
    if (value !== undefined) {
      this.value = value || '';
    }
  }

  registerOnChange(fn: any): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: any): void {
    this.onTouch = fn;
  }

  setDisabledState?(isDisabled: boolean): void {
    this.disabled = isDisabled;
  }

  onInput(event: Event): void {
    const val = (event.target as HTMLTextAreaElement).value;
    this.value = val;
    this.onChange(val);
    this.onTouch();
  }

  get textClass(): string {
    switch (this.codeColor) {
      case 'emerald': return 'text-emerald-400';
      case 'indigo': return 'text-indigo-400';
      case 'amber': return 'text-amber-400';
      case 'slate': return 'text-slate-400';
      default: return 'text-emerald-400';
    }
  }
}
