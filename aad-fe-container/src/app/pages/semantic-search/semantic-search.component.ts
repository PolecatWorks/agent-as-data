import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatCardModule } from '@angular/material/card';
import { MatChipsModule } from '@angular/material/chips';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { RouterModule, Router } from '@angular/router';
import { HttpClient } from '@angular/common/http';
import { catchError, finalize } from 'rxjs/operators';
import { of } from 'rxjs';

export interface SemanticSearchResult {
  id: string;
  entity_type: string;
  name: string;
  description: string;
  tags: string[];
  similarity_score: number;
}

@Component({
  selector: 'app-semantic-search',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
    MatCardModule,
    MatChipsModule,
    MatProgressBarModule,
    RouterModule
  ],
  templateUrl: './semantic-search.component.html',
  styleUrls: ['./semantic-search.component.scss']
})
export class SemanticSearchComponent {
  query = '';
  results: SemanticSearchResult[] = [];
  isSearching = false;
  hasSearched = false;

  private http = inject(HttpClient);
  private router = inject(Router);

  onSearch(): void {
    if (!this.query.trim()) return;

    this.isSearching = true;
    this.hasSearched = true;

    this.http.post<SemanticSearchResult[]>('/api/v1/search/semantic', { query: this.query, limit: 10 })
      .pipe(
        catchError(err => {
          console.error('Search error:', err);
          return of([]);
        }),
        finalize(() => this.isSearching = false)
      )
      .subscribe(res => {
        this.results = res || [];
      });
  }

  getScoreColor(score: number): string {
    if (score >= 0.8) return 'primary';
    if (score >= 0.5) return 'accent';
    return 'warn';
  }

  viewDetails(result: SemanticSearchResult): void {
    const routeType = result.entity_type;
    this.router.navigate(['/detail'], { queryParams: { id: result.id, type: result.entity_type } });
  }
}
