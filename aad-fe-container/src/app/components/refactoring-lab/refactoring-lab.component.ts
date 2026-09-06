import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';
import { ApiService } from '../../services/api.service';
import { ConceptGuideComponent, ConceptTabMapping } from '../concept-guide/concept-guide.component';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';

@Component({
  selector: 'app-refactoring-lab',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatMenuModule,
    RouterModule,
    ConceptGuideComponent
  ],
  templateUrl: './refactoring-lab.component.html',
  styleUrl: './refactoring-lab.component.scss'
})
export class RefactoringLabComponent implements OnInit {
  isScanning: boolean = false;
  clusters: any[] = [];
  redundantAgents: string[] = [];
  deliberateContradictions: any[] = [];

  menuItems = APP_NAV_MENU_ITEMS;

  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'hub',
      iconColor: 'text-purple-600',
      title: '1. Overlap Cluster Detection',
      description: 'Semantic clustering finding duplicated or closely related agent prompts across the workspace.'
    },
    {
      icon: 'content_cut',
      iconColor: 'text-indigo-600',
      title: '2. Redundant Skill Pruning',
      description: 'Flags overlapping capabilities across skills to keep agent prompts lean and deterministic.'
    },
    {
      icon: 'gavel',
      iconColor: 'text-red-500',
      title: '3. Invariant Conflict Analysis',
      description: 'Pre-flight compiler verification detecting contradictory behavioral invariants.'
    }
  ];

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
