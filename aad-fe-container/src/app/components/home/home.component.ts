import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';

import { APP_NAV_MENU_ITEMS, NavMenuItem } from '../../models/navigation';

export interface BusinessPillar {
  step: number;
  title: string;
  badge: string;
  icon: string;
  lead: string;
  description: string;
  keyPoints: string[];
  actionText: string;
  path: string;
  accentClass: string;
  bgClass: string;
  borderClass: string;
}

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [
    CommonModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatMenuModule,
    RouterModule
  ],
  templateUrl: './home.component.html',
  styleUrl: './home.component.scss'
})
export class HomeComponent {
  menuItems: NavMenuItem[] = APP_NAV_MENU_ITEMS;

  pillars: BusinessPillar[] = [
    {
      step: 1,
      title: 'Company Knowledge Base',
      badge: '1. Institutional Memory',
      icon: 'library_books',
      lead: 'Teach the AI your business guidelines and operating rules',
      description: 'Upload documents, guidelines, and operating playbooks. The system creates a single searchable memory that AI teammates consult before answering questions or taking action.',
      keyPoints: [
        'Central searchable company memory',
        'Cross-department domain context',
        'Eliminates generic AI hallucinations'
      ],
      actionText: 'Explore Knowledge Base',
      path: '/knowledge-inspector',
      accentClass: 'from-blue-500 to-indigo-600',
      bgClass: 'bg-blue-50/50',
      borderClass: 'border-blue-200 hover:border-blue-400'
    },
    {
      step: 2,
      title: 'Job Roles & Safety Rules (Traits)',
      badge: '2. Enforceable Contracts',
      icon: 'verified',
      lead: 'Set certified roles and unbreakable policies with Traits',
      description: 'Think of Traits like verified job certifications. They define what tools the AI is allowed to touch, what company policies it must never violate, and what data protection guardrails stay active.',
      keyPoints: [
        'Approved tools & system access only',
        'Unbreakable corporate policy rules',
        'Built-in data & password protection'
      ],
      actionText: 'Manage Trait Rules',
      path: '/traits',
      accentClass: 'from-emerald-500 to-teal-600',
      bgClass: 'bg-emerald-50/50',
      borderClass: 'border-emerald-200 hover:border-emerald-400'
    },
    {
      step: 3,
      title: 'Project Workbenches',
      badge: '3. Daily Collaboration',
      icon: 'chat',
      lead: 'Collaborate with AI teammates in dedicated project rooms',
      description: 'Your team\'s active digital workspace. Chat with AI colleagues, review files, draft documents, and let the AI perform safe, multi-turn tasks on your projects.',
      keyPoints: [
        'Dedicated project file directories',
        'Continuous conversation threads',
        'Safe stop & cancel anytime'
      ],
      actionText: 'Open Project Workbench',
      path: '/workbench',
      accentClass: 'from-indigo-500 to-purple-600',
      bgClass: 'bg-indigo-50/50',
      borderClass: 'border-indigo-200 hover:border-indigo-400'
    },
    {
      step: 4,
      title: 'AI Testing Studio',
      badge: '4. Sandbox Preview',
      icon: 'bug_report',
      lead: 'Test and preview AI teammate responses before going live',
      description: 'Try out prompts with different AI roles in a real-time sandbox. Observe how they reason and ensure high-quality, policy-compliant outputs before deploying them to your team.',
      keyPoints: [
        'Real-time output streaming',
        'Inspect prompt guidelines & rules',
        'Verify behavior before production'
      ],
      actionText: 'Launch Testing Studio',
      path: '/interactive-testing',
      accentClass: 'from-amber-500 to-orange-600',
      bgClass: 'bg-amber-50/50',
      borderClass: 'border-amber-200 hover:border-amber-400'
    }
  ];
}
