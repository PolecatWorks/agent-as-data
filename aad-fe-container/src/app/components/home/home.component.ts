import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [
    CommonModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatMenuModule,
    RouterModule
  ],
  templateUrl: './home.component.html',
  styleUrl: './home.component.scss'
})
export class HomeComponent {
  menuItems = [
    { label: 'Home', path: '/home', icon: 'home' },
    { label: 'Agents Registry', path: '/agents', icon: 'smart_toy' },
    { label: 'Traits Registry', path: '/traits', icon: 'verified' },
    { label: 'Skills Registry', path: '/skills', icon: 'extension' },
    { label: 'Tools Registry', path: '/tools', icon: 'dns' },
    { label: 'Testing Studio', path: '/interactive-testing', icon: 'bug_report' },
    { label: 'Workbench', path: '/workbench', icon: 'chat' },
    { label: 'Network Graph', path: '/network-visualizer', icon: 'account_tree' },
    { label: 'Refactoring Lab', path: '/refactoring-lab', icon: 'build_circle' },
    { label: 'Knowledge Inspector', path: '/knowledge-inspector', icon: 'library_books' }
  ];

  constructor() {}
}
