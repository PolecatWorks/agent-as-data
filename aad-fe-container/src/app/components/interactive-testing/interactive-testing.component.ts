import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';
import { RouterModule } from '@angular/router';
import { forkJoin, of } from 'rxjs';
import { catchError } from 'rxjs/operators';
import { ApiService, Agent, Skill } from '../../services/api.service';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';

export type TestEntityType = 'agent' | 'skill';

export interface TestEntity {
  id: string;
  name: string;
  type: TestEntityType;
  description?: string;
  definition?: string;
  tags?: string[];
  version?: string;
  attached_skills?: string[];
  attached_tools?: string[];
  model?: string;
  owner_id?: string;
  implements_traits?: string[];
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
    MatTooltipModule,
    RouterModule
  ],
  templateUrl: './interactive-testing.component.html',
  styleUrl: './interactive-testing.component.scss'
})
export class InteractiveTestingComponent implements OnInit {
  // Navigation Menu
  menuItems = APP_NAV_MENU_ITEMS;

  // Data
  agents: Agent[] = [];
  skills: Skill[] = [];

  // Entity selector
  allEntities: TestEntity[] = [];
  searchQuery: string = '';
  selectedEntity: TestEntity | null = null;
  isSidebarCollapsed: boolean = false;

  // Prompt Inspector State
  isPromptExpanded: boolean = true;
  copiedPrompt: boolean = false;

  // Model Selection (Local Ollama)
  availableModels: string[] = [
    'qwen2.5-coder:14b',
    'qwen2.5-coder:7b',
    'llama3.2:3b',
    'llama3.1',
    'mistral',
    'deepseek-r1'
  ];
  selectedModel: string = 'qwen2.5-coder:14b';

  // Execution
  promptInput: string = '';
  webhookUrl: string = '';
  isExecuting: boolean = false;
  executionOutput: string = '';
  finalOutput: string = '';

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
          this.selectEntity(this.allEntities[0]);
        }
        this.isLoading = false;
      },
      error: () => { this.isLoading = false; }
    });
  }

  toggleSidebar(): void {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  togglePromptExpanded(): void {
    this.isPromptExpanded = !this.isPromptExpanded;
  }

  selectModel(model: string): void {
    this.selectedModel = model;
  }

  copyPrompt(text: string): void {
    if (!text) return;
    navigator.clipboard.writeText(text);
    this.copiedPrompt = true;
    setTimeout(() => this.copiedPrompt = false, 2000);
  }

  buildEntityList(): void {
    this.allEntities = [
      ...this.agents.map(a => ({
        id: a.id,
        name: a.name,
        type: 'agent' as TestEntityType,
        description: a.description,
        definition: a.agent_definition || '',
        tags: a.tags,
        version: a.current_version,
        attached_skills: a.attached_skills || [],
        attached_tools: a.attached_tools || [],
        model: a.model || 'qwen2.5-coder:14b',
        owner_id: a.owner_id,
        implements_traits: a.implements_traits || []
      })),
      ...this.skills.map(s => ({
        id: s.id!,
        name: s.name,
        type: 'skill' as TestEntityType,
        description: s.description,
        definition: s.definition || '',
        tags: s.tags,
        version: s.current_version,
        attached_skills: s.attached_skills || [],
        attached_tools: s.attached_tools || [],
        model: 'qwen2.5-coder:14b',
        owner_id: s.owner_id,
        implements_traits: s.implements_traits || []
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
    if (entity.model) {
      this.selectedModel = entity.model;
    }
  }

  runExecution(): void {
    if (!this.promptInput.trim() || !this.selectedEntity) return;
    this.isExecuting = true;
    this.finalOutput = '';
    this.executionOutput = `[1/3] Initializing Rig execution runtime for ${this.selectedEntity.type} '${this.selectedEntity.name}'...\n` +
      `[2/3] Connecting to Ollama runtime (model: ${this.selectedModel})...\n` +
      `[3/3] Ingesting system instructions and evaluating guardrails...\n`;

    const targetId = this.selectedEntity.id;

    this.apiService.executeAgent(targetId, this.promptInput, this.webhookUrl || undefined, this.selectedModel).subscribe({
      next: (res) => {
        this.isExecuting = false;
        this.finalOutput = res.output;
        this.executionOutput += `\n[Status: ${res.status.toUpperCase()}]\n` +
          `Execution ID: ${res.execution_id}\n` +
          `Model: ${this.selectedModel}\n` +
          `\n--- Streamed LLM Output ---\n` +
          res.output;
      },
      error: (err) => {
        this.isExecuting = false;
        this.executionOutput += `\n[EXECUTION FAILED]: ${err.error || err.message || 'Server error or guardrail rejection'}`;
      }
    });
  }
}
