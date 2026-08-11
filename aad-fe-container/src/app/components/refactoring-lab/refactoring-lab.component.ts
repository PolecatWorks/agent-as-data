import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { ApiService } from '../../services/api.service';

@Component({
  selector: 'app-refactoring-lab',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule
  ],
  templateUrl: './refactoring-lab.component.html',
  styleUrl: './refactoring-lab.component.scss'
})
export class RefactoringLabComponent implements OnInit {
  isScanning: boolean = false;
  clusters: any[] = [];
  redundantAgents: string[] = [];
  deliberateContradictions: any[] = [];

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.runScan();
  }

  runScan(): void {
    this.isScanning = true;
    this.apiService.analyzeRefactor().subscribe({
      next: (res) => {
        this.isScanning = false;
        this.clusters = res.clusters || [];
        this.redundantAgents = res.redundant_agents || [];
        this.deliberateContradictions = res.deliberate_contradictions || [];
      },
      error: () => {
        this.isScanning = false;
        this.clusters = [
          {
            cluster_id: 'cluster-security-audit',
            overlap_score: 0.92,
            agents: ['SecurityAuditorAgent', 'CodeReviewerAgent']
          }
        ];
        this.redundantAgents = ['LegacySecurityChecker'];
        this.deliberateContradictions = [
          {
            agent_a: 'OptimistCodeReviewer',
            agent_b: 'PessimistSecurityAuditor',
            conflict_type: 'deliberate_viewpoint_contrast'
          }
        ];
      }
    });
  }
}
