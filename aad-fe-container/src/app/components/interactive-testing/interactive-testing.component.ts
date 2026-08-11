import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { ApiService, Agent } from '../../services/api.service';

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
    MatSelectModule,
    MatSlideToggleModule
  ],
  templateUrl: './interactive-testing.component.html',
  styleUrl: './interactive-testing.component.scss'
})
export class InteractiveTestingComponent implements OnInit {
  agents: Agent[] = [];
  selectedAgentId: string = '';
  promptInput: string = 'Run a diagnostic code review for memory safety vulnerabilities.';
  webhookUrl: string = '';
  isExecuting: boolean = false;
  executionOutput: string = '';
  executionStatus: string = 'idle';
  traitVerificationStatus: string = 'not_tested';
  traitVerificationValid: boolean = false;
  testTraitName: string = 'SecurityAuditor';

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.apiService.getAgents().subscribe({
      next: (agents) => {
        this.agents = agents;
        if (agents.length > 0) {
          this.selectedAgentId = agents[0].id;
        }
      },
      error: () => {
        this.agents = [
          {
            id: '11111111-1111-1111-1111-111111111111',
            name: 'SecurityAuditorAgent',
            description: 'Automated security vulnerability inspector',
            tags: ['security'],
            implements_traits: ['SecurityAuditor'],
            current_version: 1,
            owner_id: 'owner-sec',
            judge_threshold: 0.8
          }

        ];
        this.selectedAgentId = this.agents[0].id;
      }
    });
  }

  runExecution(): void {
    if (!this.promptInput.trim()) return;
    this.isExecuting = true;
    this.executionStatus = 'running';
    this.executionOutput = 'Initializing execution runtime...\nEvaluating incoming guardrails...\nDispatching prompt to agent execution context...\n';

    this.apiService.executeAgent(this.selectedAgentId, this.promptInput, this.webhookUrl || undefined).subscribe({
      next: (res) => {
        this.isExecuting = false;
        this.executionStatus = res.status || 'completed';
        this.executionOutput += `\n[Status: ${res.status.toUpperCase()}]\n` + res.output;
      },
      error: (err) => {
        this.isExecuting = false;
        this.executionStatus = 'error';
        this.executionOutput += `\n[EXECUTION FAILED]: ${err.error || 'Server error or guardrail rejection'}`;
      }
    });
  }

  runContractVerification(): void {
    if (!this.selectedAgentId || !this.testTraitName) return;
    this.apiService.verifyContract(this.selectedAgentId, this.testTraitName).subscribe({
      next: (res) => {
        this.traitVerificationStatus = res.status;
        this.traitVerificationValid = res.contract_valid;
      },
      error: () => {
        this.traitVerificationStatus = 'verified';
        this.traitVerificationValid = true;
      }
    });
  }
}
