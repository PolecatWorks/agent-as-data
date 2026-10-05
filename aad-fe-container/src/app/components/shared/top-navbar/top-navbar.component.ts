import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';
import { ConceptGuideComponent, ConceptTabMapping } from '../../concept-guide/concept-guide.component';
import { MatButtonModule } from '@angular/material/button';

export interface ConceptGuideConfig {
  badge: string;
  title: string;
  icon: string;
  triggerLabel: string;
  analogyTitle: string;
  analogyText: string;
  tabMappings: ConceptTabMapping[];
  detailLink: string;
  detailLinkLabel: string;
  testIdPrefix: string;
  accentColor?: 'indigo' | 'purple' | 'blue' | 'emerald' | 'amber' | 'cyan' | 'slate';
}

@Component({
  selector: 'app-top-navbar',
  standalone: true,
  imports: [CommonModule, MatIconModule, MatMenuModule, MatButtonModule, RouterModule, ConceptGuideComponent],
  templateUrl: './top-navbar.component.html',
  styleUrls: ['./top-navbar.component.scss']
})
export class TopNavbarComponent {
  @Input() workspaceTitle: string = '';
  @Input() workspaceIcon: string = '';
  @Input() conceptGuideConfig: ConceptGuideConfig | null = null;
  @Output() menuToggle = new EventEmitter<void>();
}
