import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { ApiService } from '../../services/api.service';

@Component({
  selector: 'app-mcp-manager',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatSelectModule,
    MatSnackBarModule
  ],
  templateUrl: './mcp-manager.component.html',
  styleUrl: './mcp-manager.component.scss'
})
export class McpManagerComponent implements OnInit {
  searchQuery: string = '';
  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;
  selectedServer: any | null = null;

  serverForm: any = {
    server_name: '',
    transport_type: 'sse',
    url: '',
    description: '',
    tags: []
  };

  newTag: string = '';
  isRegistering: boolean = false;

  registeredServers: any[] = [
    {
      id: '00000000-0000-0000-0000-000000000001',
      server_name: 'github-mcp-server',
      transport_type: 'sse',
      url: 'http://localhost:3000/sse',
      tools_count: 8,
      last_synced: '2 mins ago',
      tags: ['github', 'vcs']
    },
    {
      id: '00000000-0000-0000-0000-000000000002',
      server_name: 'postgres-mcp-server',
      transport_type: 'stdio',
      url: 'npx -y @modelcontextprotocol/server-postgres postgresql://localhost/db',
      tools_count: 5,
      last_synced: '1 hour ago',
      tags: ['database', 'sql']
    }
  ];

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
    this.apiService.getMcpServers().subscribe({
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
            tags: tags,
            description: description
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
        this.snackBar.open("Failed to load registered MCP servers from backend, using stubs.", "Close", { duration: 3000 });
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
      transport_type: server.transport_type,
      url: server.url || '',
      description: server.description || '',
      tags: server.tags ? [...server.tags] : []
    };
    this.router.navigate(['/mcp-servers', server.id], {
      queryParams: keepEdit ? { edit: 'true' } : {}
    });
  }

  startNewServer(): void {
    this.selectedServer = null;
    this.isEditing = true;
    this.showDeleteConfirm = false;
    this.serverForm = {
      server_name: '',
      transport_type: 'sse',
      url: '',
      description: '',
      tags: []
    };
    // Removed navigation to avoid flickering when already on the same route.
  }

  enableEdit(): void {
    this.isEditing = true;
    if (this.selectedServer) {
      this.router.navigate(['/mcp-servers', this.selectedServer.id], { queryParams: { edit: 'true' } });
    } else {
      this.router.navigate(['/mcp-servers'], { queryParams: { edit: 'true' } });
    }
  }

  cancelEdit(): void {
    if (this.selectedServer) {
      this.selectServer(this.selectedServer, false);
    } else {
      this.isEditing = false;
      this.router.navigate(['/mcp-servers'], { queryParams: {} });
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

    this.apiService.registerMcpServer(
      this.serverForm.server_name,
      this.serverForm.transport_type,
      {
        url: this.serverForm.url,
        tags: this.serverForm.tags,
        description: this.serverForm.description
      }
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
          description: this.serverForm.description
        };

        const idx = this.registeredServers.findIndex(s => s.server_name === res.server_name);
        if (idx >= 0) {
          this.registeredServers[idx] = newServer;
        } else {
          this.registeredServers.push(newServer);
        }
        this.snackBar.open(`Successfully registered MCP server: ${res.server_name}`, 'Close', { duration: 3000 });
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
          tags: [...this.serverForm.tags]
        };

        const idx = this.registeredServers.findIndex(s => s.server_name === this.serverForm.server_name);
        if (idx >= 0) {
          this.registeredServers[idx] = fallbackServer;
        } else {
          this.registeredServers.push(fallbackServer);
        }
        this.snackBar.open(`Registered MCP server: ${this.serverForm.server_name} (using stubs)`, 'Close', { duration: 3000 });
        this.selectServer(fallbackServer);
      }
    });
  }

  deleteServer(): void {
    if (!this.selectedServer) return;
    const id = this.selectedServer.id;
    this.apiService.deleteMcpServer(id).subscribe({
      next: () => {
        this.registeredServers = this.registeredServers.filter(s => s.id !== id);
        this.showDeleteConfirm = false;
        this.isEditing = false;
        this.snackBar.open(`Removed MCP server`, 'Close', { duration: 3000 });
        if (this.registeredServers.length > 0) {
          this.selectServer(this.registeredServers[0]);
        } else {
          this.selectedServer = null;
          this.router.navigate(['/mcp-servers'], { queryParams: {} });
        }
      },
      error: (err: any) => {
        this.snackBar.open(`Failed to remove MCP server: ${err.message || err}`, 'Close', { duration: 3000 });
      }
    });
  }
}
