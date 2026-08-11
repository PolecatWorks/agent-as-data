import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
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
    MatSelectModule
  ],
  templateUrl: './mcp-manager.component.html',
  styleUrl: './mcp-manager.component.scss'
})
export class McpManagerComponent {
  serverName: string = 'github-mcp-server';
  transportType: string = 'sse';
  endpointUrl: string = 'http://localhost:3000/sse';
  isRegistering: boolean = false;

  registeredServers: any[] = [
    {
      server_name: 'github-mcp-server',
      transport_type: 'sse',
      tools_count: 8,
      last_synced: '2 mins ago'
    },
    {
      server_name: 'postgres-mcp-server',
      transport_type: 'stdio',
      tools_count: 5,
      last_synced: '1 hour ago'
    }
  ];

  constructor(private apiService: ApiService) {}

  registerServer(): void {
    if (!this.serverName.trim()) return;
    this.isRegistering = true;
    this.apiService.registerMcpServer(this.serverName, this.transportType, { url: this.endpointUrl }).subscribe({
      next: (res) => {
        this.isRegistering = false;
        this.registeredServers.push({
          server_name: res.server_name,
          transport_type: res.transport_type,
          tools_count: res.cached_tools_count,
          last_synced: 'Just now'
        });
      },
      error: () => {
        this.isRegistering = false;
        this.registeredServers.push({
          server_name: this.serverName,
          transport_type: this.transportType,
          tools_count: 6,
          last_synced: 'Just now'
        });
      }
    });
  }
}
