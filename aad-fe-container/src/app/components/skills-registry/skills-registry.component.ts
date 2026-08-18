import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatSelectModule } from '@angular/material/select';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';
import { ApiService, Skill } from '../../services/api.service';
import { marked } from 'marked';

@Component({
  selector: 'app-skills-registry',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatChipsModule,
    MatTooltipModule,
    MatSnackBarModule,
    MatSelectModule
  ],
  templateUrl: './skills-registry.component.html',
  styleUrl: './skills-registry.component.scss'
})
export class SkillsRegistryComponent implements OnInit {
  skills: Skill[] = [];
  selectedSkill: Skill | null = null;
  searchQuery: string = '';
  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;

  // Editor fields
  skillForm: Partial<Skill> = {
    name: '',
    description: '',
    definition: '',
    tags: [],
    attached_skills: [],
    attached_mcp_servers: [],
    input_schema: {},
    output_schema: {},
    implementation: {}
  };
  inputSchemaStr: string = '{}';
  outputSchemaStr: string = '{}';
  implementationStr: string = '{}';

  newTagInput: string = '';
  allMcpServers: any[] = [];
  skillSearchQuery: string = '';
  mcpSearchQuery: string = '';

  constructor(
    private apiService: ApiService,
    private route: ActivatedRoute,
    private router: Router,
    private snackBar: MatSnackBar,
    private sanitizer: DomSanitizer
  ) {}

  ngOnInit(): void {
    this.loadSkills();
    this.loadMcpServers();
    this.route.params.subscribe(params => {
      if (params['id']) {
        this.loadSkill(params['id']);
      }
    });
    this.route.queryParams.subscribe(queryParams => {
      this.isEditing = queryParams['edit'] === 'true';
    });
  }

  loadMcpServers(): void {
    this.apiService.getMcpServers().subscribe({
      next: (servers) => {
        this.allMcpServers = servers || [];
      }
    });
  }

  loadSkills(): void {
    this.apiService.getSkills().subscribe({
      next: (skills) => {
        this.skills = skills;
        const routeId = this.route.snapshot.params['id'];
        if (!routeId && this.skills.length > 0) {
          this.selectSkill(this.skills[0]);
        }
      },
      error: (err) => {
        this.snackBar.open(`Error loading skills: ${err.message || err}`, 'Close', { duration: 3000 });
      }
    });
  }

  loadSkill(id: string): void {
    this.apiService.getSkill(id).subscribe({
      next: (skill) => {
        const isEdit = this.route.snapshot.queryParams['edit'] === 'true';
        this.selectSkill(skill, isEdit);
      },
      error: (err) => {
        this.snackBar.open(`Error fetching skill details: ${err.message || err}`, 'Close', { duration: 3000 });
      }
    });
  }

  selectSkill(skill: Skill, keepEdit = false): void {
    this.selectedSkill = skill;
    this.isEditing = keepEdit;
    this.showDeleteConfirm = false;
    this.skillForm = {
      ...skill,
      attached_skills: skill.attached_skills ? [...skill.attached_skills] : [],
      attached_mcp_servers: skill.attached_mcp_servers ? [...skill.attached_mcp_servers] : []
    };
    this.inputSchemaStr = JSON.stringify(skill.input_schema || {}, null, 2);
    this.outputSchemaStr = JSON.stringify(skill.output_schema || {}, null, 2);
    this.implementationStr = JSON.stringify(skill.implementation || {}, null, 2);
    this.router.navigate(['/skills-registry', skill.id], {
      queryParams: keepEdit ? { edit: 'true' } : {}
    });
  }

  startNewSkill(): void {
    this.selectedSkill = null;
    this.isEditing = true;
    this.skillForm = {
      name: '',
      description: '',
      definition: '',
      tags: [],
      current_version: 1,
      attached_skills: [],
      attached_mcp_servers: [],
      owner_id: '00000000-0000-0000-0000-000000000000',
      input_schema: {},
      output_schema: {},
      implementation: {}
    };
    this.inputSchemaStr = '{}';
    this.outputSchemaStr = '{}';
    this.implementationStr = '{}';
    this.router.navigate(['/skills-registry'], { queryParams: { edit: 'true' } });
  }

  enableEdit(): void {
    this.isEditing = true;
    if (this.selectedSkill) {
      this.router.navigate(['/skills-registry', this.selectedSkill.id], { queryParams: { edit: 'true' } });
    } else {
      this.router.navigate(['/skills-registry'], { queryParams: { edit: 'true' } });
    }
  }

  cancelEdit(): void {
    if (this.selectedSkill) {
      this.selectSkill(this.selectedSkill, false);
    } else {
      this.isEditing = false;
      this.router.navigate(['/skills-registry'], { queryParams: {} });
    }
  }

  addTag(): void {
    const val = this.newTagInput.trim();
    if (val && this.skillForm.tags && !this.skillForm.tags.includes(val)) {
      this.skillForm.tags.push(val);
      this.newTagInput = '';
    }
  }

  removeTag(tag: string): void {
    if (this.skillForm.tags) {
      this.skillForm.tags = this.skillForm.tags.filter(t => t !== tag);
    }
  }

  saveSkill(): void {
    try {
      this.skillForm.input_schema = JSON.parse(this.inputSchemaStr);
      this.skillForm.output_schema = JSON.parse(this.outputSchemaStr);
      this.skillForm.implementation = JSON.parse(this.implementationStr);
    } catch (e: any) {
      this.snackBar.open(`JSON Parse Error: ${e.message}`, 'Close', { duration: 3000 });
      return;
    }

    if (this.selectedSkill && this.selectedSkill.id) {
      this.apiService.updateSkill(this.selectedSkill.id, this.skillForm as Skill).subscribe({
        next: (updated) => {
          this.snackBar.open('Skill updated successfully!', 'Close', { duration: 2000 });
          this.loadSkills();
          this.selectSkill(updated);
        },
        error: (err) => {
          this.snackBar.open(`Update failed: ${err.message || err}`, 'Close', { duration: 3000 });
        }
      });
    } else {
      this.apiService.createSkill(this.skillForm).subscribe({
        next: (created) => {
          this.snackBar.open('Skill created successfully!', 'Close', { duration: 2000 });
          this.loadSkills();
          if (created.id) {
            this.loadSkill(created.id);
          }
        },
        error: (err) => {
          this.snackBar.open(`Creation failed: ${err.message || err}`, 'Close', { duration: 3000 });
        }
      });
    }
  }

  confirmDeleteState(): void {
    this.showDeleteConfirm = true;
  }

  cancelDelete(): void {
    this.showDeleteConfirm = false;
  }

  executeDelete(): void {
    if (this.selectedSkill && this.selectedSkill.id) {
      this.apiService.deleteSkill(this.selectedSkill.id).subscribe({
        next: () => {
          this.snackBar.open('Skill deleted successfully.', 'Close', { duration: 2000 });
          this.selectedSkill = null;
          this.isEditing = false;
          this.showDeleteConfirm = false;
          this.loadSkills();
          this.router.navigate(['/skills-registry']);
        },
        error: (err) => {
          this.snackBar.open(`Deletion failed: ${err.message || err}`, 'Close', { duration: 3000 });
        }
      });
    }
  }

  promoteToAgent(): void {
    if (this.selectedSkill && this.selectedSkill.id) {
      this.apiService.promoteSkill(this.selectedSkill.id).subscribe({
        next: (agent) => {
          this.snackBar.open(`Successfully promoted to Agent: ${agent.name}`, 'Close', { duration: 3000 });
          this.router.navigate(['/agent-registry', agent.id]);
        },
        error: (err) => {
          this.snackBar.open(`Promotion failed: ${err.message || err}`, 'Close', { duration: 3000 });
        }
      });
    }
  }

  getFilteredSkills(): Skill[] {
    const query = this.searchQuery.toLowerCase().trim();
    if (!query) return this.skills;
    return this.skills.filter(s =>
      s.name.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query) ||
      s.tags.some(t => t.toLowerCase().includes(query))
    );
  }

  getRenderedMarkdown(text: string): SafeHtml {
    if (!text) return '';
    try {
      const rawHtml = marked.parse(text) as string;
      return this.sanitizer.bypassSecurityTrustHtml(rawHtml);
    } catch (e) {
      return text;
    }
  }

  getAvailableSkillsToAttach(): Skill[] {
    return this.skills.filter(s => s.id !== this.selectedSkill?.id && !(this.skillForm.attached_skills || []).includes(s.id || ''));
  }

  getAvailableMcpToAttach(): any[] {
    return this.allMcpServers.filter(m => !(this.skillForm.attached_mcp_servers || []).includes(m.id));
  }

  getFilteredAvailableSkills(): Skill[] {
    const q = this.skillSearchQuery.toLowerCase().trim();
    const available = this.getAvailableSkillsToAttach();
    if (!q) return available;
    return available.filter(s => s.name.toLowerCase().includes(q) || (s.description && s.description.toLowerCase().includes(q)));
  }

  getFilteredAvailableMcp(): any[] {
    const q = this.mcpSearchQuery.toLowerCase().trim();
    const available = this.getAvailableMcpToAttach();
    if (!q) return available;
    return available.filter(m => m.server_name.toLowerCase().includes(q));
  }

  attachSkill(id: string): void {
    if (!this.skillForm.attached_skills) {
      this.skillForm.attached_skills = [];
    }
    if (id && !this.skillForm.attached_skills.includes(id)) {
      this.skillForm.attached_skills.push(id);
      this.skillSearchQuery = '';
    }
  }

  detachSkill(id: string): void {
    if (this.skillForm.attached_skills) {
      this.skillForm.attached_skills = this.skillForm.attached_skills.filter(i => i !== id);
    }
  }

  attachMcp(id: string): void {
    if (!this.skillForm.attached_mcp_servers) {
      this.skillForm.attached_mcp_servers = [];
    }
    if (id && !this.skillForm.attached_mcp_servers.includes(id)) {
      this.skillForm.attached_mcp_servers.push(id);
      this.mcpSearchQuery = '';
    }
  }

  detachMcp(id: string): void {
    if (this.skillForm.attached_mcp_servers) {
      this.skillForm.attached_mcp_servers = this.skillForm.attached_mcp_servers.filter(i => i !== id);
    }
  }

  getSkillName(id: string): string {
    const s = this.skills.find(x => x.id === id);
    return s ? s.name : id;
  }

  getMcpName(id: string): string {
    const m = this.allMcpServers.find(x => x.id === id);
    return m ? m.server_name : id;
  }

  getMcpDescription(id: string): string {
    const m = this.allMcpServers.find(x => x.id === id);
    if (m) {
      if (m.description) return m.description;
      let desc = `Transport: ${m.transport_type}`;
      if (m.endpoint_config && m.endpoint_config.url) {
        desc += ` — URL: ${m.endpoint_config.url}`;
      }
      return desc;
    }
    return 'No details available';
  }
}
