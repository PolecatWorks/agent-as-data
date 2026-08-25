import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatSelectModule } from '@angular/material/select';
import { MatTabsModule } from '@angular/material/tabs';
import { MatMenuModule } from '@angular/material/menu';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';
import { ApiService, Skill, TraitContract } from '../../services/api.service';
import { forkJoin } from 'rxjs';
import { marked } from 'marked';

@Component({
  selector: 'app-skills-registry',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    RouterModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatChipsModule,
    MatTooltipModule,
    MatSnackBarModule,
    MatSelectModule,
    MatTabsModule,
    MatMenuModule
  ],
  templateUrl: './skills-registry.component.html',
  styleUrl: './skills-registry.component.scss'
})
export class SkillsRegistryComponent implements OnInit {
  isSidebarCollapsed = false;

  menuItems = [
    { path: '/workbench', icon: 'dashboard', label: 'Workbench' },
    { path: '/network', icon: 'hub', label: 'Knowledge Base' },
    { path: '/skills', icon: 'extension', label: 'Skills' },
    { path: '/traits', icon: 'psychology', label: 'Traits' },
    { path: '/tools', icon: 'dns', label: 'Tools' },
    { path: '/agents', icon: 'smart_toy', label: 'Agents' },
    { path: '/testing', icon: 'science', label: 'Testing' }
  ];

  toggleSidebar() {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }
  skills: Skill[] = [];
  selectedSkill: Skill | null = null;
  searchQuery: string = '';
  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;

  traitContracts: TraitContract[] = [];
  registeredTraitsCatalog: string[] = [];
  traitSearchQuery: string = '';

  // Editor fields
  skillForm: Partial<Skill> = {
    name: '',
    description: '',
    definition: '',
    tags: [],
    implements_traits: [],
    uses_traits: [],
    attached_skills: [],
    attached_tools: [],
    input_schema: {},
    output_schema: {},
    implementation: {}
  };
  inputSchemaStr: string = '{}';
  outputSchemaStr: string = '{}';
  implementationStr: string = '{}';

  newTagInput: string = '';
  allTools: any[] = [];
  skillSearchQuery: string = '';
  toolSearchQuery: string = '';

  constructor(
    private apiService: ApiService,
    private route: ActivatedRoute,
    private router: Router,
    private snackBar: MatSnackBar,
    private sanitizer: DomSanitizer
  ) {}

  ngOnInit(): void {
    this.loadSkills();
    this.loadTools();
    this.loadTraits();
    this.route.params.subscribe(params => {
      if (params['id']) {
        this.loadSkill(params['id']);
      }
    });
    this.route.queryParams.subscribe(queryParams => {
      this.isEditing = queryParams['edit'] === 'true';
    });
  }

  loadTools(): void {
    this.apiService.getTools().subscribe({
      next: (servers) => {
        this.allTools = (servers || []).map(s => {
          // Ensure endpoint_config is an object (may be returned as a JSON string)
          let endpoint = s.endpoint_config;
          if (endpoint && typeof endpoint === 'string') {
            try {
              endpoint = JSON.parse(endpoint);
            } catch (e) {
              console.warn('Failed to parse endpoint_config JSON', e);
              endpoint = {};
            }
          }
          const tags = endpoint && endpoint.tags ? endpoint.tags : [];
          const description = endpoint && endpoint.description ? endpoint.description : '';
          return {
            ...s,
            endpoint_config: endpoint,
            tags,
            description,
          };
        });
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
      attached_tools: skill.attached_tools ? [...skill.attached_tools] : [],
      implements_traits: skill.implements_traits ? [...skill.implements_traits] : [],
      uses_traits: skill.uses_traits ? [...skill.uses_traits] : []
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
      implements_traits: [],
    uses_traits: [],
      current_version: '1.0.0',
      attached_skills: [],
      attached_tools: [],
      owner_id: '00000000-0000-0000-0000-000000000000',
      input_schema: {},
      output_schema: {},
      implementation: {}
    };
    this.inputSchemaStr = '{}';
    this.outputSchemaStr = '{}';
    this.implementationStr = '{}';
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

  getAvailableToolsToAttach(): any[] {
    return this.allTools.filter(m => !(this.skillForm.attached_tools || []).includes(m.id));
  }

  getFilteredAvailableSkills(): Skill[] {
    const q = this.skillSearchQuery.toLowerCase().trim();
    const available = this.getAvailableSkillsToAttach();
    if (!q) return available;
    return available.filter(s => s.name.toLowerCase().includes(q) || (s.description && s.description.toLowerCase().includes(q)));
  }

  getFilteredAvailableTools(): any[] {
    const q = this.toolSearchQuery.toLowerCase().trim();
    const available = this.getAvailableToolsToAttach();
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

  attachTool(id: string): void {
    if (!this.skillForm.attached_tools) {
      this.skillForm.attached_tools = [];
    }
    if (id && !this.skillForm.attached_tools.includes(id)) {
      this.skillForm.attached_tools.push(id);
      this.toolSearchQuery = '';
    }
  }

  detachTool(id: string): void {
    if (this.skillForm.attached_tools) {
      this.skillForm.attached_tools = this.skillForm.attached_tools.filter(i => i !== id);
    }
  }

  getSkillName(id: string): string {
    const s = this.skills.find(x => x.id === id);
    return s ? s.name : id;
  }

  getToolName(id: string): string {
    const m = this.allTools.find(x => x.id === id);
    return m ? m.server_name : id;
  }
  getToolDescription(id: string): string {
    const m = this.allTools.find(x => x.id === id);
    if (m) {
      if (m.description && m.description.trim().length > 0) {
        return m.description;
      }
      if (m.endpoint_config && m.endpoint_config.description) {
        return m.endpoint_config.description;
      }
      let desc = `Transport: ${m.transport_type}`;
      if (m.endpoint_config && m.endpoint_config.url) {
        desc += ` — URL: ${m.endpoint_config.url}`;
      }
      return desc;
    }
    return 'No details available';
  }

  loadTraits(): void {
    this.apiService.getTraits().subscribe({
      next: (listPages: any) => {
        const ids = listPages.ids || [];
        if (ids.length > 0) {
          const obs = ids.map((id: string) => this.apiService.getTrait(id));
          (forkJoin(obs) as any).subscribe({
            next: (fullTraits: TraitContract[]) => {
              if (fullTraits && fullTraits.length > 0) {
                this.traitContracts = fullTraits;
                this.registeredTraitsCatalog = fullTraits.map(t => t.name);
              }
            },
            error: () => {
              console.error("Failed to load details for traits.");
            }
          });
        }
      },
      error: () => {
        console.error("Failed to load traits from backend.");
      }
    });
  }

  get filteredTraitsCatalog(): string[] {
    const attached = this.skillForm.implements_traits || [];
    const available = this.registeredTraitsCatalog.filter(t => !attached.includes(t));
    if (!this.traitSearchQuery.trim()) {
      return available;
    }
    const q = this.traitSearchQuery.toLowerCase().trim();
    return available.filter(t => t.toLowerCase().includes(q));
  }

  getTraitDescription(traitName: string): string {
    const trait = this.traitContracts.find(t => t.name === traitName);
    return trait && trait.description ? trait.description : 'No description available';
  }

  attachTraitFromCatalog(trait: string): void {
    if (!this.skillForm.implements_traits) this.skillForm.implements_traits = [];
    if (!this.skillForm.implements_traits.includes(trait)) {
      this.skillForm.implements_traits.push(trait);
    }
  }

  removeTrait(trait: string): void {
    if (this.skillForm.implements_traits) {
      this.skillForm.implements_traits = this.skillForm.implements_traits.filter((t: string) => t !== trait);
    }
  }

}
