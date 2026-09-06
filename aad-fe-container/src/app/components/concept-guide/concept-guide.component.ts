import { Component, Input, HostListener, ChangeDetectorRef, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { MatIconModule } from '@angular/material/icon';

export interface ConceptTabMapping {
  icon: string;
  iconColor: string;
  title: string;
  description: string;
}

@Component({
  selector: 'app-concept-guide',
  standalone: true,
  imports: [CommonModule, RouterModule, MatIconModule],
  templateUrl: './concept-guide.component.html',
  styleUrls: ['./concept-guide.component.scss']
})
export class ConceptGuideComponent {
  private cdr = inject(ChangeDetectorRef);

  @Input() badge: string = '';
  @Input() title: string = '';
  @Input() icon: string = 'verified';
  @Input() triggerLabel: string = 'What is this?';
  @Input() analogyTitle: string = 'The Analogy';
  @Input() analogyText: string = '';
  @Input() tabMappings: ConceptTabMapping[] = [];
  @Input() detailLink: string = '/detail';
  @Input() detailLinkLabel: string = 'Explore Architecture Blueprint';
  @Input() testIdPrefix: string = 'concept-guide';
  @Input() accentColor: 'indigo' | 'purple' | 'blue' | 'emerald' | 'amber' | 'cyan' | 'slate' = 'indigo';

  isOpen = false;
  isPinned = false;
  private debounceTimer: any = null;

  toggle(event?: MouseEvent): void {
    if (event) {
      event.stopPropagation();
    }
    this.clearTimer();
    if (this.isPinned) {
      this.isPinned = false;
      this.isOpen = false;
    } else {
      this.isPinned = true;
      this.isOpen = true;
    }
    this.cdr.markForCheck();
  }

  show(): void {
    this.clearTimer();
    this.isOpen = true;
    this.cdr.markForCheck();
  }

  hide(): void {
    if (!this.isPinned) {
      this.clearTimer();
      this.debounceTimer = setTimeout(() => {
        if (!this.isPinned) {
          this.isOpen = false;
          this.cdr.markForCheck();
        }
      }, 250);
    }
  }

  close(event?: MouseEvent): void {
    if (event) {
      event.stopPropagation();
    }
    this.clearTimer();
    this.isPinned = false;
    this.isOpen = false;
    this.cdr.markForCheck();
  }

  @HostListener('document:keydown.escape')
  onEscapePress(): void {
    if (this.isOpen) {
      this.close();
    }
  }

  private clearTimer(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
  }
}
