import {
  Component,
  OnInit,
  ChangeDetectorRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';

// Angular Material
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSelectModule } from '@angular/material/select';
import { MatTabsModule } from '@angular/material/tabs';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';

// Shared
import { CodeEditorTextareaComponent } from '../../components/shared/code-editor-textarea/code-editor-textarea.component';

import { ApiService } from '../../services/api.service';


@Component({
  selector: 'app-tool-detail',
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
    MatSelectModule,
    MatTabsModule,
    MatSnackBarModule,
    CodeEditorTextareaComponent,
    RouterModule,
  ],
  templateUrl: './tool-detail.component.html',
  styleUrl: './tool-detail.component.scss',
})
export class ToolDetailComponent implements OnInit {
  selectedServer: any | null = null;
  serverState: any | null = null;
  serverForm: any = {
    server_name: '',
    transport_type: 'sse',
    url: '',
    tags: ['tool'],
    description: '',
    owner_id: '00000000-0000-0000-0000-000000000000',
  };

  newTag: string = '';
  isEditing: boolean = false;
  isRegistering: boolean = false;
  showDeleteConfirm: boolean = false;

  // Tool Verification State
  activeTestTool: any = null;
  testArgsJson: string = '{}';
  isExecutingTest: boolean = false;
  testResult: any = null;
  testError: string | null = null;

  constructor(
    private snackBar: MatSnackBar,
    private apiService: ApiService,
    private route: ActivatedRoute,
    private router: Router,
    private cdr: ChangeDetectorRef,
  ) {}

  ngOnInit(): void {
    this.route.paramMap.subscribe(params => {
      const id = params.get('id');
      if (this.route.snapshot.routeConfig?.path === "new") {
        this.createNewServer();
      } else if (id) {
        this.selectServerById(id);
      }
    });
    this.route.queryParams.subscribe((queryParams) => {
      if (this.route.snapshot.routeConfig?.path !== "new") {
        this.isEditing = queryParams['edit'] === 'true';
      }
    });
  }

  selectServerById(id: string): void {
    this.apiService.getTools().subscribe({
      next: (servers: any[]) => {
        const server = servers.find(s => s.id === id);
        if (server) {
          this.selectServer(server, this.isEditing);
        } else {
          this.snackBar.open(`Server ${id} not found`, 'Close', { duration: 3000 });
          this.router.navigate(['/tools']);
        }
      },
      error: () => {
        this.snackBar.open(`Failed to load server ${id}`, 'Close', { duration: 3000 });
      }
    });
  }

  selectServer(server: any, keepEdit = false): void {
    this.selectedServer = server;
    this.serverForm = { ...server, tags: server.tags || [] };
    this.showDeleteConfirm = false;
    this.fetchServerState(server.id);

    if (!keepEdit) {
      this.isEditing = false;
      this.router.navigate(['/tools', server.id]);
    } else {
      this.router.navigate(['/tools', server.id], { queryParams: { edit: 'true' } });
    }
  }

  createNewServer(): void {
    this.selectedServer = null;
    this.serverForm = {
      server_name: '',
      transport_type: 'sse',
      url: '',
      tags: ['tool'],
      description: '',
      owner_id: '00000000-0000-0000-0000-000000000000',
    };
    this.serverState = null;
    this.isEditing = true;
    this.showDeleteConfirm = false;
  }

  fetchServerState(id: string): void {
    this.serverState = null;
    this.apiService.syncTool(id).subscribe({
      next: (state: any) => {
        this.serverState = state;
        this.cdr.detectChanges();
      },
      error: () => {
        this.serverState = { status: 'offline', tools: [] };
        this.cdr.detectChanges();
      },
    });
  }

  syncServerStatus(): void {
    if (!this.selectedServer) return;
    this.fetchServerState(this.selectedServer.id);
  }

  openToolTester(tool: any): void {
    this.activeTestTool = tool;
    this.testResult = null;
    this.testError = null;

    let sampleArgs: any = {};
    if (tool.inputSchema && tool.inputSchema.properties) {
      for (const [key, prop] of Object.entries<any>(
        tool.inputSchema.properties,
      )) {
        if (prop.type === 'string') {
          sampleArgs[key] = '';
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

    this.apiService
      .testTool(this.selectedServer.id, this.activeTestTool.name, parsedArgs)
      .subscribe({
        next: (res) => {
          this.isExecutingTest = false;
          this.testResult = res;
        },
        error: (err) => {
          this.isExecutingTest = false;
          this.testError =
            err.error?.message ||
            err.error ||
            err.message ||
            'Execution request failed';
        },
      });
  }

  getToolProperties(
    tool: any,
  ): Array<{
    name: string;
    type: string;
    description: string;
    required: boolean;
  }> {
    if (!tool.inputSchema || !tool.inputSchema.properties) return [];
    const requiredList = Array.isArray(tool.inputSchema.required)
      ? tool.inputSchema.required
      : [];
    return Object.entries<any>(tool.inputSchema.properties).map(
      ([key, val]) => ({
        name: key,
        type: val.type || 'any',
        description: val.description || '',
        required: requiredList.includes(key),
      }),
    );
  }

  enableEdit(): void {
    this.isEditing = true;
    if (this.selectedServer) {
      this.router.navigate(['/tools', this.selectedServer.id], {
        queryParams: { edit: 'true' },
      });
    }
  }

  cancelEdit(): void {
    if (this.selectedServer) {
      this.selectServer(this.selectedServer, false);
    } else {
      this.isEditing = false;
      this.router.navigate(['/tools']);
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
    this.serverForm.tags = this.serverForm.tags.filter(
      (t: string) => t !== tag,
    );
  }

  registerServer(): void {
    if (!this.serverForm.server_name.trim()) return;
    this.isRegistering = true;

    this.apiService
      .registerTool(
        this.serverForm.server_name,
        this.serverForm.transport_type,
        {
          url: this.serverForm.url,
          tags: this.serverForm.tags,
          description: this.serverForm.description,
        },
        this.serverForm.owner_id,
      )
      .subscribe({
        next: (res) => {
          this.isRegistering = false;
          this.snackBar.open(`Successfully registered tool: ${res.server_name}`, 'Close', { duration: 3000 });
          this.selectServer(res as any, false);
        },
        error: (err: any) => {
          this.isRegistering = false;
          this.snackBar.open(`Failed to register tool: ${err.message || err}`, 'Close', { duration: 3000 });
        },
      });
  }

  deleteServer(): void {
    if (!this.selectedServer) return;
    const id = this.selectedServer.id;
    this.apiService.deleteTool(id).subscribe({
      next: () => {
        this.showDeleteConfirm = false;
        this.isEditing = false;
        this.selectedServer = null;
        this.snackBar.open(`Removed tool`, 'Close', { duration: 3000 });
        this.router.navigate(['/tools']);
      },
      error: (err: any) => {
        this.snackBar.open(`Failed to remove tool: ${err.message || err}`, 'Close', { duration: 3000 });
      },
    });
  }
}
