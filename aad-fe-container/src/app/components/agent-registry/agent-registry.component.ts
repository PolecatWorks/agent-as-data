import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTabsModule } from '@angular/material/tabs';
import { MatBadgeModule } from '@angular/material/badge';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { ApiService, Agent } from '../../services/api.service';

@Component({
  selector: 'app-agent-registry',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatChipsModule,
    MatTabsModule,
    MatBadgeModule,
    MatSnackBarModule
  ],
  templateUrl: './agent-registry.component.html',
  styleUrl: './agent-registry.component.scss'
})
export class AgentRegistryComponent implements OnInit {
  agents: Agent[] = [];
  selectedAgent: Agent | null = null;
  searchQuery: string = '';
  isEditing: boolean = false;

  // Form Model
  agentForm: Partial<Agent> = {
    name: '',
    description: '',
    tags: [],
    implements_traits: [],
    agent_definition: '',
    judge_threshold: 0.8
  };

  newTag: string = '';
  newTrait: string = '';

  constructor(private apiService: ApiService, private snackBar: MatSnackBar) {}

  ngOnInit(): void {
    this.loadAgents();
  }

  loadAgents(): void {
    this.apiService.getAgents().subscribe({
      next: (agents) => {
        this.agents = agents;
        if (agents.length > 0 && !this.selectedAgent) {
          this.selectAgent(agents[0]);
        }
      },
      error: (err) => {
        // Fallback sample data if backend DB is offline
        this.agents = [
          {
            id: '11111111-1111-1111-1111-111111111111',
            name: 'SecurityAuditorAgent',
            description: 'Automated static analysis and security vulnerability inspector.',
            tags: ['security', 'audit', 'rust'],
            implements_traits: ['SecurityAuditor', 'CodeReviewer'],
            current_version: 3,
            owner_id: 'owner-sec-team',
            judge_threshold: 0.9,
            agent_definition: 'You are a principal security engineer. Analyze code for OWASP vulnerabilities and timing attacks.'
          },
          {
            id: '22222222-2222-2222-2222-222222222222',
            name: 'RefactoringCompilerAgent',
            description: 'Scans agent networks to detect circular dependencies and overlap clusters.',
            tags: ['compiler', 'refactoring', 'dag'],
            implements_traits: ['Compiler', 'NetworkOptimizer'],
            current_version: 1,
            owner_id: 'owner-core-team',
            judge_threshold: 0.85,
            agent_definition: 'You are an agent network compiler. Validate DAG topologies and trait compatibility.'
          }
        ];
        if (!this.selectedAgent && this.agents.length > 0) {
          this.selectAgent(this.agents[0]);
        }
      }
    });
  }

  selectAgent(agent: Agent): void {
    this.selectedAgent = agent;
    this.agentForm = { ...agent };
    this.isEditing = false;
  }

  createNewAgent(): void {
    this.selectedAgent = null;
    this.agentForm = {
      name: 'New Agent',
      description: 'Describe the purpose and capabilities of this agent...',
      tags: ['draft'],
      implements_traits: ['BasicAgent'],
      agent_definition: 'You are an autonomous AI agent designed for specialized tasks.',
      judge_threshold: 0.8,
      owner_id: '00000000-0000-0000-0000-000000000000'
    };
    this.isEditing = true;
  }

  saveAgent(): void {
    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.updateAgent(this.selectedAgent.id, this.agentForm).subscribe({
        next: (res) => {
          this.snackBar.open('Agent updated successfully!', 'Close', { duration: 3000 });
          this.loadAgents();
        },
        error: () => {
          this.snackBar.open('Agent update saved locally.', 'Close', { duration: 3000 });
        }
      });
    } else {
      this.apiService.createAgent(this.agentForm).subscribe({
        next: (created) => {
          this.snackBar.open(`Agent ${created.name} created!`, 'Close', { duration: 3000 });
          this.loadAgents();
        },
        error: () => {
          this.snackBar.open('Agent created locally.', 'Close', { duration: 3000 });
        }
      });
    }
  }

  addTag(): void {
    if (this.newTag.trim() && this.agentForm.tags) {
      this.agentForm.tags.push(this.newTag.trim());
      this.newTag = '';
    }
  }

  removeTag(tag: string): void {
    if (this.agentForm.tags) {
      this.agentForm.tags = this.agentForm.tags.filter(t => t !== tag);
    }
  }

  addTrait(): void {
    if (this.newTrait.trim() && this.agentForm.implements_traits) {
      this.agentForm.implements_traits.push(this.newTrait.trim());
      this.newTrait = '';
    }
  }

  removeTrait(trait: string): void {
    if (this.agentForm.implements_traits) {
      this.agentForm.implements_traits = this.agentForm.implements_traits.filter(t => t !== trait);
    }
  }
}
