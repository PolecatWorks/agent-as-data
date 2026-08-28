import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatSliderModule } from '@angular/material/slider';
import { MatCardModule } from '@angular/material/card';
import { MatChipsModule } from '@angular/material/chips';
import { ApiService } from '../../services/api.service';

@Component({
  selector: 'app-agent-context',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatSliderModule,
    MatCardModule,
    MatChipsModule
  ],
  templateUrl: './agent-context.component.html',
  styleUrls: ['./agent-context.component.scss']
})
export class AgentContextComponent {
  searchQuery: string = '';
  traceDepth: number = 5;
  searchResults: any[] = [];
  isSearching: boolean = false;

  constructor(private apiService: ApiService) {}

  onSearch(): void {
    if (!this.searchQuery.trim()) return;
    this.isSearching = true;
    this.apiService.searchAgentContext(this.searchQuery, this.traceDepth).subscribe({
      next: (results) => {
        this.searchResults = results;
        this.isSearching = false;
      },
      error: (err) => {
        console.error('Error searching context:', err);
        this.isSearching = false;
      }
    });
  }
}
