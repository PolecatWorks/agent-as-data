import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';
import { forkJoin, of } from 'rxjs';
import { catchError } from 'rxjs/operators';
import { ApiService, Agent, Skill } from '../../services/api.service';

export type TestEntityType = 'agent' | 'skill';

export interface TestEntity {
  id: string;
  name: string;
  type: TestEntityType;
  description?: string;
  tags?: string[];
  version?: string;
}

@Component({
  selector: 'app-interactive-testing',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatMenuModule,
    RouterModule
  ],
  templateUrl: './interactive-testing.component.html',
  styleUrl: './interactive-testing.component.scss'
})
export class InteractiveTestingComponent implements OnInit {
  // Navigation Menu
  menuItems = [
    { label: 'Home', path: '/home', icon: 'home' },
    { label: 'Agents Registry', path: '/agents', icon: 'smart_toy' },
    { label: 'Traits Registry', path: '/traits', icon: 'verified' },
    { label: 'Skills Registry', path: '/skills', icon: 'extension' },
    { label: 'Tools Registry', path: '/tools', icon: 'dns' },
    { label: 'Testing Studio', path: '/interactive-testing', icon: 'bug_report' },
    { label: 'Workbench', path: '/workbench', icon: 'chat' },
    { label: 'Network Graph', path: '/network-visualizer', icon: 'account_tree' },
    { label: 'Refactoring Lab', path: '/refactoring-lab', icon: 'build_circle' },
    { label: 'Knowledge Inspector', path: '/knowledge-inspector', icon: 'library_books' }
  ];
  // Data
  agents: Agent[] = [];
  skills: Skill[] = [];

  // Entity selector (mirrors network visualizer pattern)
  allEntities: TestEntity[] = [];
  searchQuery: string = '';
  selectedEntity: TestEntity | null = null;

  // Execution
  promptInput: string = 'Run a diagnostic code review for memory safety vulnerabilities.';
  webhookUrl: string = '';
  isExecuting: boolean = false;
  executionOutput: string = '';

  // State
  isLoading: boolean = false;

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.isLoading = true;
    forkJoin({
      agents: this.apiService.getAgents().pipe(catchError(() => of([] as Agent[]))),
      skills: this.apiService.getSkills().pipe(catchError(() => of([] as Skill[])))
    }).subscribe({
      next: ({ agents, skills }) => {
        this.agents = agents as Agent[];
        this.skills = skills as Skill[];
        this.buildEntityList();
        if (this.allEntities.length > 0) {
          this.selectedEntity = this.allEntities[0];
        }
        this.isLoading = false;
      },
      error: () => { this.isLoading = false; }
    });
  }

  buildEntityList(): void {
    this.allEntities = [
      ...this.agents.map(a => ({
        id: a.id,
        name: a.name,
        type: 'agent' as TestEntityType,
        description: a.description,
        tags: a.tags,
        version: a.current_version
      })),
      ...this.skills.map(s => ({
        id: s.id!,
        name: s.name,
        type: 'skill' as TestEntityType,
        description: s.description,
        tags: s.tags,
        version: s.current_version
      }))
    ];
  }

  getFilteredEntities(): TestEntity[] {
    const q = this.searchQuery.toLowerCase().trim();
    if (!q) return this.allEntities;
    return this.allEntities.filter(e =>
      e.name.toLowerCase().includes(q) ||
      (e.description && e.description.toLowerCase().includes(q)) ||
      (e.tags && e.tags.some(t => t.toLowerCase().includes(q))) ||
      e.type.toLowerCase().includes(q)
    );
  }

  selectEntity(entity: TestEntity): void {
    this.selectedEntity = entity;
  }

  getEntityIcon(type: TestEntityType): string {
    return type === 'agent' ? '🤖' : '⚡';
  }

  runExecution(): void {
    if (!this.promptInput.trim() || !this.selectedEntity) return;
    this.isExecuting = true;
    this.executionOutput = 'Initializing execution runtime...\nEvaluating incoming guardrails...\nDispatching prompt to agent execution context...\n';

    // Only agents can be directly executed; skills are executed via their parent agent
    const agentId = this.selectedEntity.type === 'agent'
      ? this.selectedEntity.id
      : this.agents.find(a => (a.attached_skills || []).includes(this.selectedEntity!.id))?.id || '';

    this.apiService.executeAgent(agentId, this.promptInput, this.webhookUrl || undefined).subscribe({
      next: (res) => {
        this.isExecuting = false;
        this.executionOutput += `\n[Status: ${res.status.toUpperCase()}]\n` + res.output;
      },
      error: (err) => {
        this.isExecuting = false;
        this.executionOutput += `\n[EXECUTION FAILED]: ${err.error || 'Server error or guardrail rejection'}`;
      }
    });
  }
}
