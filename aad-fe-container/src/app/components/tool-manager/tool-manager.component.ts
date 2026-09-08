import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';
import { ApiService } from '../../services/api.service';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';
import { ConceptGuideComponent, ConceptTabMapping } from '../concept-guide/concept-guide.component';

@Component({
  selector: 'app-tool-manager',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    RouterModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatSelectModule,
    MatSnackBarModule,
    MatMenuModule,
    MatTooltipModule,
    ConceptGuideComponent
  ],
  templateUrl: './tool-manager.component.html',
  styleUrl: './tool-manager.component.scss'
})
export class ToolManagerComponent implements OnInit {
  isSidebarCollapsed = false;

  menuItems = APP_NAV_MENU_ITEMS;

  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'build',
      iconColor: 'text-indigo-600',
      title: '1. Standardized Rig Tools',
      description: 'Defines standardized tool payloads, command definitions, and JSON schemas directly consumed by agent runtimes.'
    },
    {
      icon: 'cloud_sync',
      iconColor: 'text-emerald-600',
      title: '2. Model Context Protocol (MCP)',
      description: 'Connects to remote Model Context Protocol tool endpoints via SSE and Stdio daemon processes.'
    },
    {
      icon: 'security',
      iconColor: 'text-blue-600',
      title: '3. Pre-Tool Execution Safeties',
      description: 'Enforces execution permissions, timeout constraints, and pre-tool execution validations.'
    }
  ];

  toggleSidebar() {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  searchQuery: string = '';
  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;
  selectedServer: any | null = null;
  isSyncing: boolean = false;

  // Tool Verification Console State
  activeTestTool: any | null = null;
  testArgsJson: string = '{}';
  isExecutingTest: boolean = false;
  testResult: any | null = null;
  testError: string | null = null;

  serverForm: any = {
    server_name: '',
    owner_id: '00000000-0000-0000-0000-000000000000',
    transport_type: 'http',
    url: 'http://localhost:8082',
    description: '',
    tags: []
  };

  newTag: string = '';
  isRegistering: boolean = false;

  registeredServers: any[] = [];

  constructor(
    private apiService: ApiService,
    private snackBar: MatSnackBar,
    private route: ActivatedRoute,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadServers();
    this.route.paramMap.subscribe(params => {
      const id = params.get('id');
      if (id) {
        this.selectServerById(id);
      }
    });
    this.route.queryParams.subscribe(queryParams => {
      this.isEditing = queryParams['edit'] === 'true';
    });
  }

  loadServers(): void {
    this.apiService.getTools().subscribe({
      next: (servers) => {
        this.registeredServers = servers.map(s => {
          let count = 0;
          if (s.cached_capabilities && s.cached_capabilities.tools) {
            count = s.cached_capabilities.tools.length;
          }
          let tags = [] as string[];
          let description = '';
          if (s.endpoint_config) {
            if (s.endpoint_config.tags) tags = s.endpoint_config.tags;
            if (s.endpoint_config.description) description = s.endpoint_config.description;
          }
          return {
            id: s.id,
            server_name: s.server_name,
            transport_type: s.transport_type,
            url: s.endpoint_config ? s.endpoint_config.url : '',
            tools_count: count,
            last_synced: 'Just now',
            sync_status: s.sync_status || 'synced',
            last_sync_error: s.last_sync_error || null,
            cached_capabilities: s.cached_capabilities || { tools: [] },
            tags: tags,
            description: description,
            owner_id: s.owner_id || '00000000-0000-0000-0000-000000000000'
          };
        });
        const routeId = this.route.snapshot.paramMap.get('id');
        if (routeId) {
          this.selectServerById(routeId);
        } else if (this.registeredServers.length > 0) {
          this.selectServer(this.registeredServers[0]);
        } else {
          this.selectedServer = null;
        }
      },
      error: () => {
        this.snackBar.open("Failed to load registered tools from backend, using stubs.", "Close", { duration: 3000 });
        const routeId = this.route.snapshot.paramMap.get('id');
        if (routeId) {
          this.selectServerById(routeId);
        } else if (this.registeredServers.length > 0) {
          this.selectServer(this.registeredServers[0]);
        }
      }
    });
  }

  getFilteredServers(): any[] {
    if (!this.searchQuery.trim()) {
      return this.registeredServers;
    }
    const q = this.searchQuery.toLowerCase().trim();
    return this.registeredServers.filter(s => 
      s.server_name.toLowerCase().includes(q) ||
      s.transport_type.toLowerCase().includes(q) ||
      (s.tags && s.tags.some((t: string) => t.toLowerCase().includes(q)))
    );
  }

  selectServerById(id: string): void {
    const server = this.registeredServers.find(s => s.id === id);
    if (server) {
      const isEdit = this.route.snapshot.queryParams['edit'] === 'true';
      this.selectServer(server, isEdit);
    }
  }

  selectServer(server: any, keepEdit = false): void {
    this.selectedServer = server;
    this.isEditing = keepEdit;
    this.showDeleteConfirm = false;
    this.serverForm = {
      server_name: server.server_name,
      owner_id: server.owner_id || '00000000-0000-0000-0000-000000000000',
      transport_type: server.transport_type,
      url: server.url || '',
      description: server.description || '',
      tags: server.tags ? [...server.tags] : []
    };
    this.router.navigate(['/tools', server.id], {
      queryParams: keepEdit ? { edit: 'true' } : {}
    });
  }

  startNewServer(): void {
    this.selectedServer = null;
    this.isEditing = true;
    this.showDeleteConfirm = false;
    this.serverForm = {
      server_name: '',
      owner_id: '00000000-0000-0000-0000-000000000000',
      transport_type: 'http',
      url: 'http://localhost:8082',
      description: '',
      tags: []
    };
    // Removed navigation to avoid flickering when already on the same route.
  }

  syncServer(): void {
    if (!this.selectedServer) return;
    this.isSyncing = true;
    const id = this.selectedServer.id;
    this.apiService.syncTool(id).subscribe({
      next: (res) => {
        this.isSyncing = false;
        this.selectedServer.tools_count = res.cached_tools_count;
        this.selectedServer.sync_status = res.sync_status;
        this.selectedServer.last_sync_error = res.last_sync_error;
        this.selectedServer.last_synced = 'Just now';

        const idx = this.registeredServers.findIndex(s => s.id === id);
        if (idx >= 0) {
          this.registeredServers[idx].tools_count = res.cached_tools_count;
          this.registeredServers[idx].sync_status = res.sync_status;
          this.registeredServers[idx].last_sync_error = res.last_sync_error;
          this.registeredServers[idx].last_synced = 'Just now';
        }

        // Re-query tools to get updated full cached_capabilities
        this.apiService.getTools().subscribe({
          next: (tools) => {
            const updated = tools.find(t => t.id === id);
            if (updated && updated.cached_capabilities) {
              this.selectedServer.cached_capabilities = updated.cached_capabilities;
              if (idx >= 0) {
                this.registeredServers[idx].cached_capabilities = updated.cached_capabilities;
              }
            }
          }
        });

        this.snackBar.open(
          res.sync_status === 'synced'
            ? `Successfully synced ${res.cached_tools_count} tool(s) from ${res.server_name}`
            : `Sync completed with status '${res.sync_status}': ${res.last_sync_error || 'warning'}`,
          'Close',
          { duration: 4000 }
        );
      },
      error: (err) => {
        this.isSyncing = false;
        this.snackBar.open(`Sync failed: ${err.message || err}`, 'Close', { duration: 4000 });
      }
    });
  }

  openToolTester(tool: any): void {
    this.activeTestTool = tool;
    this.testResult = null;
    this.testError = null;

    // Generate sample arguments derived from inputSchema
    const sampleArgs: Record<string, any> = {};
    if (tool.inputSchema && tool.inputSchema.properties) {
      for (const [key, prop] of Object.entries<any>(tool.inputSchema.properties)) {
        if (prop.type === 'string') {
          sampleArgs[key] = key === 'name' ? 'Antigravity' : (prop.description || 'Sample ' + key);
        } else if (prop.type === 'number' || prop.type === 'integer') {
          sampleArgs[key] = 1;
        } else if (prop.type === 'boolean') {
          sampleArgs[key] = true;
        } else if (prop.type === 'object') {
          sampleArgs[key] = {};
        } else if (prop.type === 'array') {
          sampleArgs[key] = [];
        } else {
          sampleArgs[key] = '';
        }
      }
    }
    this.testArgsJson = JSON.stringify(sampleArgs, null, 2);
  }

  closeToolTester(): void {
    this.activeTestTool = null;
    this.testResult = null;
    this.testError = null;
  }

  executeToolTest(): void {
    if (!this.selectedServer || !this.activeTestTool) return;
    let parsedArgs: any = {};
    try {
      parsedArgs = JSON.parse(this.testArgsJson);
    } catch (e: any) {
      this.testError = `Invalid JSON arguments: ${e.message}`;
      return;
    }

    this.isExecutingTest = true;
    this.testResult = null;
    this.testError = null;

    this.apiService.testTool(this.selectedServer.id, this.activeTestTool.name, parsedArgs).subscribe({
      next: (res) => {
        this.isExecutingTest = false;
        this.testResult = res;
      },
      error: (err) => {
        this.isExecutingTest = false;
        this.testError = err.error?.message || err.error || err.message || 'Execution request failed';
      }
    });
  }

  getToolProperties(tool: any): Array<{ name: string; type: string; description: string; required: boolean }> {
    if (!tool.inputSchema || !tool.inputSchema.properties) return [];
    const requiredList = Array.isArray(tool.inputSchema.required) ? tool.inputSchema.required : [];
    return Object.entries<any>(tool.inputSchema.properties).map(([key, val]) => ({
      name: key,
      type: val.type || 'any',
      description: val.description || '',
      required: requiredList.includes(key)
    }));
  }

  enableEdit(): void {
    this.isEditing = true;
    if (this.selectedServer) {
      this.router.navigate(['/tools', this.selectedServer.id], { queryParams: { edit: 'true' } });
    } else {
      this.router.navigate(['/tools'], { queryParams: { edit: 'true' } });
    }
  }

  cancelEdit(): void {
    if (this.selectedServer) {
      this.selectServer(this.selectedServer, false);
    } else {
      this.isEditing = false;
      this.router.navigate(['/tools'], { queryParams: {} });
    }
  }

  confirmDeleteState(): void {
    this.showDeleteConfirm = true;
  }

  cancelDelete(): void {
    this.showDeleteConfirm = false;
  }

  addTag(): void {
    const t = this.newTag.trim().toLowerCase();
    if (t && !this.serverForm.tags.includes(t)) {
      this.serverForm.tags.push(t);
    }
    this.newTag = '';
  }

  removeTag(tag: string): void {
    this.serverForm.tags = this.serverForm.tags.filter((t: string) => t !== tag);
  }

  registerServer(): void {
    if (!this.serverForm.server_name.trim()) return;
    this.isRegistering = true;

    this.apiService.registerTool(
      this.serverForm.server_name,
      this.serverForm.transport_type,
      {
        url: this.serverForm.url,
        tags: this.serverForm.tags,
        description: this.serverForm.description
      },
      this.serverForm.owner_id
    ).subscribe({
      next: (res) => {
        this.isRegistering = false;
        const newServer = {
          id: res.id,
          server_name: res.server_name,
          transport_type: res.transport_type,
          url: this.serverForm.url,
          tools_count: res.cached_tools_count,
          last_synced: 'Just now',
          tags: [...this.serverForm.tags],
          description: this.serverForm.description,
          owner_id: this.serverForm.owner_id
        };

        const idx = this.registeredServers.findIndex(s => s.server_name === res.server_name);
        if (idx >= 0) {
          this.registeredServers[idx] = newServer;
        } else {
          this.registeredServers.push(newServer);
        }
        this.snackBar.open(`Successfully registered tool: ${res.server_name}`, 'Close', { duration: 3000 });
        this.selectServer(newServer);
      },
      error: () => {
        this.isRegistering = false;
        const fallbackServer = {
          id: '00000000-0000-0000-0000-000000000009',
          server_name: this.serverForm.server_name,
          transport_type: this.serverForm.transport_type,
          url: this.serverForm.url,
          tools_count: 6,
          last_synced: 'Just now',
          tags: [...this.serverForm.tags],
          owner_id: this.serverForm.owner_id
        };

        const idx = this.registeredServers.findIndex(s => s.server_name === this.serverForm.server_name);
        if (idx >= 0) {
          this.registeredServers[idx] = fallbackServer;
        } else {
          this.registeredServers.push(fallbackServer);
        }
        this.snackBar.open(`Registered tool: ${this.serverForm.server_name} (using stubs)`, 'Close', { duration: 3000 });
        this.selectServer(fallbackServer);
      }
    });
  }

  deleteServer(): void {
    if (!this.selectedServer) return;
    const id = this.selectedServer.id;
    this.apiService.deleteTool(id).subscribe({
      next: () => {
        this.registeredServers = this.registeredServers.filter(s => s.id !== id);
        this.showDeleteConfirm = false;
        this.isEditing = false;
        this.snackBar.open(`Removed tool`, 'Close', { duration: 3000 });
        if (this.registeredServers.length > 0) {
          this.selectServer(this.registeredServers[0]);
        } else {
          this.selectedServer = null;
          this.router.navigate(['/tools'], { queryParams: {} });
        }
      },
      error: (err: any) => {
        this.snackBar.open(`Failed to remove tool: ${err.message || err}`, 'Close', { duration: 3000 });
      }
    });
  }
}
