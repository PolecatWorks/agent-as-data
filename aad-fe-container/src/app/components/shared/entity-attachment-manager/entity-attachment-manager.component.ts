import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatTooltipModule } from '@angular/material/tooltip';

export interface AttachmentEntity {
  id: string;
  name: string;
  description?: string;
}

@Component({
  selector: 'app-entity-attachment-manager',
  standalone: true,
  imports: [CommonModule, FormsModule, MatButtonModule, MatTooltipModule],
  templateUrl: './entity-attachment-manager.component.html',
  styleUrl: './entity-attachment-manager.component.scss'
})
export class EntityAttachmentManagerComponent {
  @Input() title: string = 'Attached Entities';
  @Input() themeColor: 'indigo' | 'emerald' | 'purple' | 'teal' | 'orange' | 'blue' | 'slate' = 'indigo';
  @Input() availableEntities: AttachmentEntity[] = [];
  @Input() attachedIds: string[] = [];
  @Input() isEditing: boolean = false;

  @Output() attach = new EventEmitter<string>();
  @Output() detach = new EventEmitter<string>();

  searchQuery: string = '';

  get filteredAvailableEntities(): AttachmentEntity[] {
    const query = this.searchQuery.toLowerCase();
    return this.availableEntities.filter(entity => 
      !this.attachedIds.includes(entity.id) &&
      (entity.name.toLowerCase().includes(query) || 
       (entity.description && entity.description.toLowerCase().includes(query)))
    );
  }

  getEntityName(id: string): string {
    // Look first in availableEntities, even if it's attached it should be there, 
    // but sometimes the parent might only pass unattached ones. 
    // Ideally parent passes ALL known entities to availableEntities or a separate full dictionary.
    // Assuming parent passes ALL entities to availableEntities.
    const entity = this.availableEntities.find(e => e.id === id);
    return entity ? entity.name : id;
  }

  getEntityDescription(id: string): string {
    const entity = this.availableEntities.find(e => e.id === id);
    return entity?.description || 'No description available';
  }

  onAttach(id: string) {
    this.attach.emit(id);
  }

  onDetach(id: string) {
    this.detach.emit(id);
  }

  // Theme helper maps to ensure Tailwind doesn't purge the classes
  get containerClasses(): string {
    const maps: Record<string, string> = {
      'indigo': 'p-5 bg-indigo-50/30 rounded-xl border border-indigo-200/50 space-y-3',
      'emerald': 'p-5 bg-emerald-50/50 rounded-xl border border-emerald-200/80 space-y-4',
      'purple': 'p-5 bg-purple-50/50 rounded-xl border border-purple-200/80 space-y-4',
      'teal': 'p-5 bg-teal-50/30 rounded-xl border border-teal-200/50 space-y-3',
      'orange': 'p-5 bg-orange-50/30 rounded-xl border border-orange-200/50 space-y-3',
      'blue': 'p-5 bg-blue-50/30 rounded-xl border border-blue-200/50 space-y-3',
      'slate': 'p-5 bg-slate-50/30 rounded-xl border border-slate-200/50 space-y-3'
    };
    return maps[this.themeColor] || maps['indigo'];
  }

  get titleClasses(): string {
    const maps: Record<string, string> = {
      'indigo': 'text-sm font-bold text-indigo-950 m-0 flex items-center gap-2',
      'emerald': 'text-sm font-bold text-emerald-950 m-0 flex items-center gap-2',
      'purple': 'text-sm font-bold text-purple-950 m-0 flex items-center gap-2',
      'teal': 'text-sm font-bold text-teal-950 m-0 flex items-center gap-2',
      'orange': 'text-sm font-bold text-orange-950 m-0 flex items-center gap-2',
      'blue': 'text-sm font-bold text-blue-950 m-0 flex items-center gap-2',
      'slate': 'text-sm font-bold text-slate-950 m-0 flex items-center gap-2'
    };
    return maps[this.themeColor] || maps['indigo'];
  }
  
  get pillClasses(): string {
    const maps: Record<string, string> = {
      'indigo': 'px-3 py-1 bg-indigo-100 text-indigo-900 font-bold text-xs rounded-xl flex items-center gap-2 border border-indigo-200',
      'emerald': 'px-3 py-1 bg-emerald-100 text-emerald-800 font-bold text-xs rounded-xl flex items-center gap-2 border border-emerald-200',
      'purple': 'px-3 py-1 bg-purple-100 text-purple-900 font-bold text-xs rounded-xl flex items-center gap-2 border border-purple-200',
      'teal': 'px-3 py-1 bg-teal-100 text-teal-900 font-bold text-xs rounded-xl flex items-center gap-2 border border-teal-200',
      'orange': 'px-3 py-1 bg-orange-100 text-orange-900 font-bold text-xs rounded-xl flex items-center gap-2 border border-orange-200',
      'blue': 'px-3 py-1 bg-blue-100 text-blue-900 font-bold text-xs rounded-xl flex items-center gap-2 border border-blue-200',
      'slate': 'px-3 py-1 bg-slate-100 text-slate-900 font-bold text-xs rounded-xl flex items-center gap-2 border border-slate-200'
    };
    return maps[this.themeColor] || maps['indigo'];
  }

  get detachButtonClasses(): string {
    const maps: Record<string, string> = {
      'indigo': 'text-indigo-700 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm',
      'emerald': 'text-emerald-600 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm',
      'purple': 'text-purple-700 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm',
      'teal': 'text-teal-700 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm',
      'orange': 'text-orange-700 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm',
      'blue': 'text-blue-700 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm',
      'slate': 'text-slate-700 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm'
    };
    return maps[this.themeColor] || maps['indigo'];
  }
}
