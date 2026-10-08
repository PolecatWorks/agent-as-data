import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';

export type EntityPillTheme = 'indigo' | 'emerald' | 'amber' | 'slate' | 'red';

@Component({
  selector: 'app-entity-pill',
  standalone: true,
  imports: [CommonModule, MatIconModule],
  templateUrl: './entity-pill.component.html',
  styleUrl: './entity-pill.component.scss'
})
export class EntityPillComponent {
  @Input({ required: true }) label!: string;
  @Input() icon?: string;
  @Input() themeColor: EntityPillTheme = 'slate';
  @Input() isRemovable: boolean = false;
  
  @Output() removed = new EventEmitter<void>();
  
  get themeClasses() {
    switch (this.themeColor) {
      case 'indigo': return 'bg-indigo-50 text-indigo-700 border-indigo-200';
      case 'emerald': return 'bg-emerald-50 text-emerald-700 border-emerald-200';
      case 'amber': return 'bg-amber-50 text-amber-700 border-amber-200';
      case 'red': return 'bg-red-50 text-red-700 border-red-200';
      case 'slate': default: return 'bg-slate-100 text-slate-700 border-slate-200';
    }
  }
  
  onRemove(event: Event) {
    event.stopPropagation();
    event.preventDefault();
    this.removed.emit();
  }
}
