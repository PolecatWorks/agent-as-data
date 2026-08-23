import { Component } from '@angular/core';
import { RouterOutlet, RouterModule } from '@angular/router';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatListModule } from '@angular/material/list';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    RouterOutlet,
    RouterModule,
    MatSidenavModule,
    MatToolbarModule,
    MatListModule,
    MatIconModule,
    MatButtonModule
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss'
})
export class AppComponent {
  title = 'Agent-As-Data Studio';

  menuItems = [
    { path: '/traits-registry', icon: 'verified', label: 'Trait Contracts' },
    { path: '/tools', icon: 'dns', label: 'MCP Servers' },
    { path: '/skills-registry', icon: 'extension', label: 'Skills' },
    { path: '/agent-registry', icon: 'app_registration', label: 'Agents' },
    { path: '/interactive-testing', icon: 'bug_report', label: 'Interactive Testing Studio' },
    { path: '/network-visualizer', icon: 'account_tree', label: 'Network Graph Visualizer' },
    { path: '/refactoring-lab', icon: 'build_circle', label: 'Refactoring & Compression Lab' },
    { path: '/knowledge-inspector', icon: 'library_books', label: 'Knowledge & SPO Tuple Inspector' }
  ];

}
