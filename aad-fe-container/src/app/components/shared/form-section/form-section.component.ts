import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';

export type FormSectionTheme = 'indigo' | 'emerald' | 'amber' | 'blue' | 'purple' | 'slate';

@Component({
  selector: 'app-form-section',
  standalone: true,
  imports: [CommonModule, MatIconModule],
  templateUrl: './form-section.component.html',
  styleUrl: './form-section.component.scss'
})
export class FormSectionComponent {
  @Input({ required: true }) title!: string;
  @Input({ required: true }) icon!: string;
  @Input() description?: string;
  @Input() themeColor: FormSectionTheme = 'indigo';

  get themeClasses() {
    switch (this.themeColor) {
      case 'indigo':
        return { accentBorder: 'bg-indigo-500', iconContainer: 'bg-indigo-50 border-indigo-100', iconText: 'text-indigo-600' };
      case 'emerald':
        return { accentBorder: 'bg-emerald-500', iconContainer: 'bg-emerald-50 border-emerald-100', iconText: 'text-emerald-600' };
      case 'amber':
        return { accentBorder: 'bg-amber-500', iconContainer: 'bg-amber-50 border-amber-100', iconText: 'text-amber-600' };
      case 'blue':
        return { accentBorder: 'bg-blue-500', iconContainer: 'bg-blue-50 border-blue-100', iconText: 'text-blue-600' };
      case 'purple':
        return { accentBorder: 'bg-purple-500', iconContainer: 'bg-purple-50 border-purple-100', iconText: 'text-purple-600' };
      case 'slate':
      default:
        return { accentBorder: 'bg-slate-400', iconContainer: 'bg-slate-100 border-slate-200', iconText: 'text-slate-600' };
    }
  }
}
