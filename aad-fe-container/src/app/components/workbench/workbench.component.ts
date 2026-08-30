import { Component, OnInit, HostListener } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';
import { RouterModule, ActivatedRoute, Router } from '@angular/router';
import { ApiService, Thread, Message } from '../../services/api.service';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';

@Component({
  selector: 'app-workbench',
  standalone: true,
  imports: [CommonModule, FormsModule, MatCardModule, MatButtonModule, MatIconModule, MatMenuModule, MatTooltipModule, RouterModule],
  templateUrl: './workbench.component.html',
  styleUrl: './workbench.component.scss'
})
export class WorkbenchComponent implements OnInit {
  threads: Thread[] = [];
  activeThread: Thread | null = null;
  activeThreadMessages: Message[] = [];
  newMessageContent = '';
  searchQuery: string = '';
  isProcessing: boolean = false;

  files: string[] = [];
  selectedFile: string | null = null;
  selectedFileContent: string = '';

  isSidebarCollapsed = false;
  chatPanePercentage = 50;
  isResizing = false;

  isEditingTitle = false;
  editingTitleContent = '';

  menuItems = APP_NAV_MENU_ITEMS;

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

    const sidebarWidth = this.isSidebarCollapsed ? 64 : 256;
    const rightWorkspaceWidth = window.innerWidth - sidebarWidth;
    const mouseXInWorkspace = event.clientX - sidebarWidth;

    let newPercentage = (mouseXInWorkspace / rightWorkspaceWidth) * 100;

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
    this.loadThreadFiles();
  }

  loadThreadFiles(): void {
    if (!this.activeThread) return;
    this.apiService.listThreadFiles(this.activeThread.id).subscribe({
      next: (res) => {
        this.files = res.files;
        if (this.selectedFile && !this.files.includes(this.selectedFile)) {
          this.selectedFile = null;
          this.selectedFileContent = '';
        }
      },
      error: (err) => console.error('Failed to load files', err)
    });
  }

  selectFile(filename: string): void {
    if (!this.activeThread) return;

    if (!filename.endsWith('.txt') && !filename.endsWith('.md')) {
      alert('Only .txt and .md files are supported for editing at this time.');
      return;
    }

    this.apiService.readThreadFile(this.activeThread.id, filename).subscribe({
      next: (res) => {
        this.selectedFile = filename;
        this.selectedFileContent = res.content;
      },
      error: (err) => console.error('Failed to read file', err)
    });
  }

  createNewFile(): void {
    if (!this.activeThread) return;
    const filename = prompt('Enter new filename:');
    if (!filename) return;

    if (!filename.endsWith('.txt') && !filename.endsWith('.md')) {
      alert('Only .txt and .md files are supported at this time.');
      return;
    }

    this.apiService.writeThreadFile(this.activeThread.id, filename, '').subscribe({
      next: () => {
        this.loadThreadFiles();
        this.selectFile(filename);
      },
      error: (err) => console.error('Failed to create file', err)
    });
  }

  saveFile(): void {
    if (!this.activeThread || !this.selectedFile) return;
    this.apiService.writeThreadFile(this.activeThread.id, this.selectedFile, this.selectedFileContent).subscribe({
      next: () => {
        // Optional: show a success toast here
        console.log(`Saved ${this.selectedFile}`);
      },
      error: (err) => console.error('Failed to save file', err)
    });
  }

  deleteFile(filename: string, event: Event): void {
    event.stopPropagation();
    if (!this.activeThread) return;

    if (confirm(`Are you sure you want to delete ${filename}?`)) {
      this.apiService.deleteThreadFile(this.activeThread.id, filename).subscribe({
        next: () => {
          if (this.selectedFile === filename) {
            this.selectedFile = null;
            this.selectedFileContent = '';
          }
          this.loadThreadFiles();
        },
        error: (err) => console.error('Failed to delete file', err)
      });
    }
  }

  sendMessage(): void {
    if (!this.newMessageContent.trim() || !this.activeThread || this.isProcessing) {
      return;
    }

    const content = this.newMessageContent;
    this.newMessageContent = '';
    this.isProcessing = true;

    this.apiService.createMessage(this.activeThread.id, 'user', content).subscribe({
      next: (message) => {
        this.activeThreadMessages.push(message);
        this.isProcessing = false;
        // Also reload files as the LLM might have modified them
        this.loadThreadFiles();
      },
      error: (err) => {
        console.error('Failed to send message', err);
        this.isProcessing = false;
      }
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
      this.activeThread.title = newTitle;
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
