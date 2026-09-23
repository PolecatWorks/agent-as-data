import { Component, OnInit, ElementRef, AfterViewInit } from '@angular/core';
import { CommonModule, TitleCasePipe } from '@angular/common';
import { RouterModule } from '@angular/router';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { NavMenuItem, APP_NAV_MENU_ITEMS } from '../../models/navigation';
import { ApiService } from '../../services/api.service';
import { Network } from 'vis-network';
import { DataSet } from 'vis-data';
import { forkJoin } from 'rxjs';

@Component({
  selector: 'app-knowledge-explorer',
  standalone: true,
  imports: [CommonModule, RouterModule, MatIconModule, MatButtonModule, MatMenuModule, TitleCasePipe],
  templateUrl: './knowledge-explorer.component.html',
  styleUrl: './knowledge-explorer.component.scss'
})
export class KnowledgeExplorerComponent implements OnInit, AfterViewInit {
  menuItems: NavMenuItem[] = APP_NAV_MENU_ITEMS;
  selectedNode: any = null;
  network: any;
  nodes = new DataSet([]);
  edges = new DataSet([]);

  constructor(private el: ElementRef, private apiService: ApiService) {}

  ngOnInit(): void {}

  ngAfterViewInit(): void {
    this.initGraph();
    this.loadData();
  }

  initGraph(): void {
    const container = this.el.nativeElement.querySelector('#network-graph');
    const data = {
      nodes: this.nodes,
      edges: this.edges
    };
    const options = {
      nodes: {
        shape: 'dot',
        size: 20,
        font: {
          size: 14,
          color: '#334155'
        },
        borderWidth: 2
      },
      edges: {
        width: 1,
        color: { color: '#cbd5e1', highlight: '#94a3b8' },
        smooth: {
          type: 'continuous'
        }
      },
      physics: {
        forceAtlas2Based: {
          gravitationalConstant: -26,
          centralGravity: 0.005,
          springLength: 230,
          springConstant: 0.18
        },
        maxVelocity: 146,
        solver: 'forceAtlas2Based',
        timestep: 0.35,
        stabilization: { iterations: 150 }
      },
      interaction: {
        hover: true,
        tooltipDelay: 200,
        zoomView: true
      }
    };

    this.network = new Network(container, data, options);

    this.network.on('selectNode', (params: any) => {
      const nodeId = params.nodes[0];
      const node = this.nodes.get(nodeId);
      if (node) {
        // Find raw data
        this.selectedNode = node.rawData;
      }
    });

    this.network.on('deselectNode', () => {
      this.selectedNode = null;
    });
  }

  loadData(): void {
    forkJoin({
      agents: this.apiService.getAgents(),
      skills: this.apiService.getSkills(),
      tools: this.apiService.getTools(),
      traits: this.apiService.getTraits(),
      knowledge: this.apiService.getKnowledgeNodes()
    }).subscribe(results => {
      const newNodes: any[] = [];
      const newEdges: any[] = [];

      // Add Agents
      results.agents.forEach(agent => {
        newNodes.push({
          id: agent.id,
          label: agent.name,
          color: { background: '#f3e8ff', border: '#a855f7', highlight: { background: '#e9d5ff', border: '#9333ea' } },
          rawData: { ...agent, type: 'agent' }
        });
      });

      // Add Skills
      results.skills.forEach((skill: any) => {
        newNodes.push({
          id: skill.id,
          label: skill.name,
          color: { background: '#fef3c7', border: '#f59e0b', highlight: { background: '#fde68a', border: '#d97706' } },
          rawData: { ...skill, type: 'skill' }
        });

        // Edges: Agent -> Skill
        if (skill.attached_agents) {
          skill.attached_agents.forEach((agentId: string) => {
             newEdges.push({ from: agentId, to: skill.id, arrows: 'to', dashes: true });
          });
        }
      });

      // Add Tools
      results.tools.forEach((tool: any) => {
        newNodes.push({
          id: tool.id,
          label: tool.name || tool.server_name,
          color: { background: '#dbeafe', border: '#3b82f6', highlight: { background: '#bfdbfe', border: '#2563eb' } },
          rawData: { ...tool, type: 'tool', name: tool.name || tool.server_name }
        });
      });

      // Add Traits
      if (results.traits && Array.isArray((results.traits as any).items || results.traits)) {
        ((results.traits as any).items || (results.traits as any)).forEach((trait: any) => {
          newNodes.push({
            id: trait.id,
            label: trait.name,
            color: { background: '#d1fae5', border: '#10b981', highlight: { background: '#a7f3d0', border: '#059669' } },
            rawData: { ...trait, type: 'trait' }
          });
        });

        // Try to connect agents to traits if they have implements_traits
        results.agents.forEach(agent => {
          if (agent.implements_traits) {
            agent.implements_traits.forEach(traitName => {
               // Find trait by name
               const trait = ((results.traits as any).items || (results.traits as any)).find((t: any) => t.name === traitName);
               if (trait) {
                 newEdges.push({ from: agent.id, to: trait.id, arrows: 'to', dashes: true });
               }
            });
          }
        });
      }

      // Add Knowledge Nodes
      results.knowledge.forEach(node => {
        newNodes.push({
          id: node.id,
          label: node.title || node.topic,
          shape: 'square',
          color: { background: '#e0e7ff', border: '#6366f1', highlight: { background: '#c7d2fe', border: '#4f46e5' } },
          rawData: { ...node, type: 'knowledge', name: node.title || node.topic }
        });
      });

      this.nodes.clear();
      this.edges.clear();
      this.nodes.add(newNodes);
      this.edges.add(newEdges);

      this.network.fit();
    });
  }

  closePanel(): void {
    this.selectedNode = null;
    this.network.unselectAll();
  }

  getIcon(type: string): string {
    switch (type) {
      case 'agent': return 'smart_toy';
      case 'skill': return 'extension';
      case 'tool': return 'dns';
      case 'trait': return 'verified';
      case 'knowledge': return 'library_books';
      default: return 'help_outline';
    }
  }

  getIconColor(type: string): string {
    switch (type) {
      case 'agent': return 'text-purple-600';
      case 'skill': return 'text-amber-600';
      case 'tool': return 'text-blue-600';
      case 'trait': return 'text-emerald-600';
      case 'knowledge': return 'text-indigo-600';
      default: return 'text-slate-600';
    }
  }
}
