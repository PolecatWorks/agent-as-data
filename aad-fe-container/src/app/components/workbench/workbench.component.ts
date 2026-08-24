import { Component, OnInit, HostListener } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule, ActivatedRoute, Router } from '@angular/router';
import { ApiService, Thread, Message } from '../../services/api.service';

@Component({
  selector: 'app-workbench',
  standalone: true,
  imports: [CommonModule, FormsModule, MatCardModule, MatButtonModule, MatIconModule, MatMenuModule, RouterModule],
  templateUrl: './workbench.component.html',
  styleUrl: './workbench.component.scss'
})
export class WorkbenchComponent implements OnInit {
  threads: Thread[] = [];
  activeThread: Thread | null = null;
  activeThreadMessages: Message[] = [];
  newMessageContent = '';
  searchQuery: string = '';

  isSidebarCollapsed = false;
  chatPanePercentage = 50;
  isResizing = false;

  isEditingTitle = false;
  editingTitleContent = '';

  menuItems = [
    { path: '/traits-registry', icon: 'verified', label: 'Trait Contracts' },
    { path: '/tools', icon: 'dns', label: 'Tools' },
    { path: '/skills-registry', icon: 'extension', label: 'Skills' },
    { path: '/agent-registry', icon: 'app_registration', label: 'Agents' },
    { path: '/interactive-testing', icon: 'bug_report', label: 'Interactive Testing Studio' },
    { path: '/network-visualizer', icon: 'account_tree', label: 'Network Graph Visualizer' },
    { path: '/refactoring-lab', icon: 'build_circle', label: 'Refactoring & Compression Lab' },
    { path: '/knowledge-inspector', icon: 'library_books', label: 'Knowledge & SPO Tuple Inspector' },
    { path: '/workbench', icon: 'work', label: 'Workbench' }
  ];

  constructor(
    private apiService: ApiService,
    private route: ActivatedRoute,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadThreads();
  }

  toggleSidebar(): void {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  startResizing(event: MouseEvent): void {
    this.isResizing = true;
    event.preventDefault();
  }

  @HostListener('document:mousemove', ['$event'])
  onMouseMove(event: MouseEvent): void {
    if (!this.isResizing) return;
    
    // Calculate new percentage based on mouse position
    // Assuming the resizer is between the two panes in the right workspace
    // We'll need to know the width of the right workspace container, but a rough calculation based on screen width can work, 
    // or we can use a ViewChild for precise measurement. 
    // For simplicity, let's just do a rough calculation assuming sidebar is 256px when open, 64px when closed.
    const sidebarWidth = this.isSidebarCollapsed ? 64 : 256;
    const rightWorkspaceWidth = window.innerWidth - sidebarWidth;
    const mouseXInWorkspace = event.clientX - sidebarWidth;
    
    let newPercentage = (mouseXInWorkspace / rightWorkspaceWidth) * 100;
    
    // Constrain the percentage
    if (newPercentage < 20) newPercentage = 20;
    if (newPercentage > 80) newPercentage = 80;
    
    this.chatPanePercentage = newPercentage;
  }

  @HostListener('document:mouseup')
  onMouseUp(): void {
    this.isResizing = false;
  }

  getFilteredThreads(): Thread[] {
    const query = this.searchQuery.toLowerCase().trim();
    if (!query) return this.threads;
    return this.threads.filter(t => t.title.toLowerCase().includes(query));
  }

  loadThreads(): void {
    this.apiService.getThreads().subscribe({
      next: (threads) => {
        this.threads = threads;
        
        this.route.paramMap.subscribe(params => {
          const threadId = params.get('id');
          if (threadId) {
            const threadToSelect = this.threads.find(t => t.id === threadId);
            if (threadToSelect && this.activeThread?.id !== threadId) {
              this.loadThreadContent(threadToSelect);
            }
          } else if (this.threads.length > 0) {
             this.router.navigate(['/workbench', this.threads[0].id], { replaceUrl: true });
          }
        });
      },
      error: (err) => console.error('Failed to load threads', err)
    });
  }

  createNewThread(): void {
    const title = `New Conversation ${this.threads.length + 1}`;
    this.apiService.createThread(title).subscribe({
      next: (thread) => {
        this.threads.unshift(thread);
        this.selectThread(thread);
      },
      error: (err) => console.error('Failed to create thread', err)
    });
  }

  deleteThread(threadId: string, event?: Event): void {
    if (event) {
      event.stopPropagation();
    }

    if (confirm('Are you sure you want to delete this thread?')) {
      this.apiService.deleteThread(threadId).subscribe({
        next: () => {
          this.threads = this.threads.filter(t => t.id !== threadId);
          if (this.activeThread?.id === threadId) {
            this.activeThread = null;
            this.activeThreadMessages = [];
            if (this.threads.length > 0) {
              this.selectThread(this.threads[0]);
            } else {
              this.router.navigate(['/workbench']);
            }
          }
        },
        error: (err) => console.error('Failed to delete thread', err)
      });
    }
  }

  selectThread(thread: Thread): void {
    this.router.navigate(['/workbench', thread.id]);
  }

  loadThreadContent(thread: Thread): void {
    this.activeThread = thread;
    this.activeThreadMessages = [];
    this.apiService.getMessages(thread.id).subscribe({
      next: (messages) => this.activeThreadMessages = messages,
      error: (err) => console.error('Failed to load messages', err)
    });
  }

  sendMessage(): void {
    if (!this.newMessageContent.trim() || !this.activeThread) {
      return;
    }

    const content = this.newMessageContent;
    this.newMessageContent = ''; // Clear input immediately

    this.apiService.createMessage(this.activeThread.id, 'user', content).subscribe({
      next: (message) => {
        this.activeThreadMessages.push(message);
        // Normally an agent process would kick off here and respond.
        // For the sake of the UI implementation, we simply append the message.
      },
      error: (err) => console.error('Failed to send message', err)
    });
  }

  startEditingTitle(): void {
    if (this.activeThread) {
      this.isEditingTitle = true;
      this.editingTitleContent = this.activeThread.title;
    }
  }

  saveThreadTitle(): void {
    if (!this.activeThread || !this.isEditingTitle) return;
    
    const newTitle = this.editingTitleContent.trim();
    if (newTitle && newTitle !== this.activeThread.title) {
      this.activeThread.title = newTitle; // optimistic update
      this.apiService.updateThread(this.activeThread.id, newTitle).subscribe({
        next: (updatedThread) => {
          if (this.activeThread && this.activeThread.id === updatedThread.id) {
             this.activeThread.title = updatedThread.title;
             const t = this.threads.find(x => x.id === updatedThread.id);
             if (t) t.title = updatedThread.title;
          }
        },
        error: (err) => console.error('Failed to update thread title', err)
      });
    }
    this.isEditingTitle = false;
  }
  
  cancelEditingTitle(): void {
    this.isEditingTitle = false;
  }
}
