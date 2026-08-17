import { Component, ElementRef, OnInit, ViewChild, AfterViewInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { RouterModule } from '@angular/router';
import mermaid from 'mermaid';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [
    CommonModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    RouterModule
  ],
  templateUrl: './home.component.html',
  styleUrl: './home.component.scss'
})
export class HomeComponent implements OnInit, AfterViewInit {
  @ViewChild('mermaidContainer') mermaidContainer!: ElementRef;

  mermaidCode: string = `flowchart TD
    subgraph Ingestion ["1. Knowledge Ingestion & Graph Store"]
        K1["Raw Text Documents / Wikis"] -->|Chunked & Vectorized| K2["Semantic Search (pgvector)"]
        K3["Entity Relationship Tuples (SPO)"] -->|Indexed| K4["Graph Traversal Engine"]
    end

    subgraph Definition ["2. Declarative Specifications"]
        T1["Trait Contracts Registry"] -->|Implements Capability & Invariants| A1["Agent Registry & Builder"]
        A1 -->|Promoted / Demoted| S1["Managed Skills Registry"]
    end

    subgraph Validation ["3. Pre-Flight Verification & Testing"]
        A1 -->|Verify Contract / Semantic Fit| V1["DAG Compiler & Verification"]
        A1 -->|Deterministic assertions + LLM Judge| V2["Agent Unit Testing Kit"]
    end

    subgraph Execution ["4. Execution & Observability"]
        V1 & V2 -->|Safe and Validated| E1["Execution Engine (Sync/Async)"]
        E1 -->|Exposed via| M1["Native MCP Server (Stdio/SSE)"]
        E1 -->|Generates| L1["Structured Usage & Telemetry Logs"]
    end

    classDef ing fill:#EFF6FF,stroke:#3B82F6,stroke-width:2px,color:#1E3A8A;
    classDef def fill:#ECFDF5,stroke:#10B981,stroke-width:2px,color:#064E3B;
    classDef val fill:#FFFBEB,stroke:#F59E0B,stroke-width:2px,color:#78350F;
    classDef exe fill:#FDF2F8,stroke:#EC4899,stroke-width:2px,color:#701A75;

    class K1,K2,K3,K4 ing;
    class T1,A1,S1 def;
    class V1,V2 val;
    class E1,M1,L1 exe;`;

  constructor() {}

  ngOnInit(): void {
    mermaid.initialize({ startOnLoad: false, theme: 'neutral' });
  }

  ngAfterViewInit(): void {
    this.renderDiagram();
  }

  async renderDiagram(): Promise<void> {
    if (!this.mermaidContainer) return;
    try {
      const { svg } = await mermaid.render('mermaid-svg-home-diagram', this.mermaidCode);
      this.mermaidContainer.nativeElement.innerHTML = svg;
    } catch (e) {
      console.error('Mermaid render error:', e);
    }
  }
}
