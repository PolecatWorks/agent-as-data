import { Component, ElementRef, OnInit, ViewChild, AfterViewInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import mermaid from 'mermaid';

@Component({
  selector: 'app-network-visualizer',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule
  ],
  templateUrl: './network-visualizer.component.html',
  styleUrl: './network-visualizer.component.scss'
})
export class NetworkVisualizerComponent implements OnInit, AfterViewInit {
  @ViewChild('mermaidContainer') mermaidContainer!: ElementRef;

  mermaidCode: string = `flowchart TD
    Orchestrator["OrchestratorAgent (v2)"] -->|delegates| SecAudit["SecurityAuditor (v3)"]
    Orchestrator -->|delegates| Refactor["RefactoringCompiler (v1)"]
    SecAudit -->|uses trait| SecurityTrait["Trait: SecurityAuditor"]
    Refactor -->|uses trait| CompilerTrait["Trait: Compiler"]
    SecAudit -->|uses skill| SkillScan["Skill: RustMemoryScan"]`;

  constructor() {}

  ngOnInit(): void {
    mermaid.initialize({ startOnLoad: false, theme: 'default' });
  }

  ngAfterViewInit(): void {
    this.renderDiagram();
  }

  async renderDiagram(): Promise<void> {

    if (!this.mermaidContainer) return;
    try {
      const { svg } = await mermaid.render('mermaid-svg-diagram', this.mermaidCode);
      this.mermaidContainer.nativeElement.innerHTML = svg;
    } catch (e) {
      console.error('Mermaid render error:', e);
    }
  }
}
