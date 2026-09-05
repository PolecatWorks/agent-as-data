import { Component, OnInit, HostListener, ViewChild, ElementRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';
import { RouterModule, ActivatedRoute, Router } from '@angular/router';
import { ApiService, Bench, Thread, Message } from '../../services/api.service';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';

@Component({
  selector: 'app-workbench',
  standalone: true,
  imports: [CommonModule, FormsModule, MatCardModule, MatButtonModule, MatIconModule, MatMenuModule, MatTooltipModule, RouterModule],
  templateUrl: './workbench.component.html',
  styleUrl: './workbench.component.scss'
})
export class WorkbenchComponent implements OnInit {
  @ViewChild('messageInput') messageInput?: ElementRef<HTMLTextAreaElement>;
  @ViewChild('messagesContainer') messagesContainer?: ElementRef<HTMLDivElement>;

  benches: Bench[] = [];
  activeBench: Bench | null = null;
  isBenchDropdownOpen: boolean = false;
  isCreatingBenchInline: boolean = false;
  newBenchName: string = '';
  isEditingBenchName: boolean = false;
  editingBenchNameContent: string = '';
  isConfirmingDeleteBench: boolean = false;

  threads: Thread[] = [];
  activeThread: Thread | null = null;
  activeThreadMessages: Message[] = [];
  newMessageContent = '';
  searchQuery: string = '';
  isProcessing: boolean = false;

  isConfirmingDeleteThreadId: string | null = null;
  editingThreadId: string | null = null;
  editingThreadTitleContent: string = '';

  files: string[] = [];
  selectedFile: string | null = null;
  selectedFileContent: string = '';

  activeRightTab: 'files' | 'memory' = 'files';
  benchWorkingMemoryContent: string = '';
  isSavingMemory: boolean = false;
  memorySaveStatus: string = '';

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
    this.loadBenches();
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

    const sidebarWidth = this.isSidebarCollapsed ? 64 : 288;
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

  loadBenches(): void {
    this.apiService.getBenches().subscribe({
      next: (benches) => {
        this.benches = benches;
        if (this.benches.length === 0) {
          this.apiService.createBench('Default Bench', undefined, 'Initial default workspace').subscribe({
            next: (newBench) => {
              this.benches = [newBench];
              this.resolveRouting();
            },
            error: (err) => console.error('Failed to scaffold initial bench', err)
          });
        } else {
          this.resolveRouting();
        }
      },
      error: (err) => console.error('Failed to load benches', err)
    });
  }

  resolveRouting(): void {
    this.route.paramMap.subscribe(params => {
      const benchId = params.get('benchId');
      const threadId = params.get('threadId');

      if (!benchId) {
        if (this.benches.length > 0) {
          const targetBench = this.benches[0];
          this.router.navigate(['/workbench', targetBench.id], { replaceUrl: true });
        }
        return;
      }

      const foundBench = this.benches.find(b => b.id === benchId);
      if (foundBench) {
        this.activeBench = foundBench;
        this.loadBenchThreads(foundBench.id, threadId);
      } else if (this.benches.length > 0) {
        this.router.navigate(['/workbench', this.benches[0].id], { replaceUrl: true });
      }
    });
  }

  loadBenchThreads(benchId: string, targetThreadId?: string | null): void {
    this.apiService.getBenchThreads(benchId).subscribe({
      next: (threads) => {
        this.threads = threads;
        if (targetThreadId) {
          const toSelect = this.threads.find(t => t.id === targetThreadId);
          if (toSelect) {
            this.loadThreadContent(toSelect);
          } else if (this.threads.length > 0) {
            this.router.navigate(['/workbench', benchId, this.threads[0].id], { replaceUrl: true });
          }
        } else if (this.threads.length > 0) {
          this.router.navigate(['/workbench', benchId, this.threads[0].id], { replaceUrl: true });
        } else {
          this.activeThread = null;
          this.activeThreadMessages = [];
          this.loadBenchFiles();
        }
      },
      error: (err) => console.error('Failed to load bench threads', err)
    });
  }

  toggleBenchDropdown(): void {
    this.isBenchDropdownOpen = !this.isBenchDropdownOpen;
    this.isCreatingBenchInline = false;
    this.isConfirmingDeleteBench = false;
  }

  selectBench(bench: Bench): void {
    this.activeBench = bench;
    this.isBenchDropdownOpen = false;
    this.isConfirmingDeleteBench = false;
    this.router.navigate(['/workbench', bench.id]);
  }

  startCreatingBenchInline(): void {
    this.isCreatingBenchInline = true;
    this.newBenchName = '';
  }

  cancelCreatingBenchInline(): void {
    this.isCreatingBenchInline = false;
    this.newBenchName = '';
  }

  commitCreateBench(): void {
    const name = this.newBenchName.trim();
    if (!name) return;

    this.apiService.createBench(name).subscribe({
      next: (bench) => {
        this.benches.unshift(bench);
        this.isCreatingBenchInline = false;
        this.isBenchDropdownOpen = false;
        this.selectBench(bench);
      },
      error: (err) => console.error('Failed to create bench', err)
    });
  }

  startEditingBenchName(): void {
    if (!this.activeBench) return;
    this.isEditingBenchName = true;
    this.editingBenchNameContent = this.activeBench.name;
  }

  saveBenchName(): void {
    if (!this.activeBench || !this.isEditingBenchName) return;
    const name = this.editingBenchNameContent.trim();
    if (!name || name === this.activeBench.name) {
      this.isEditingBenchName = false;
      return;
    }

    this.apiService.updateBench(this.activeBench.id, name).subscribe({
      next: (updated) => {
        if (this.activeBench) {
          this.activeBench.name = updated.name;
        }
        const b = this.benches.find(x => x.id === updated.id);
        if (b) b.name = updated.name;
        this.isEditingBenchName = false;
      },
      error: (err) => {
        console.error('Failed to rename bench', err);
        this.isEditingBenchName = false;
      }
    });
  }

  cancelEditingBenchName(): void {
    this.isEditingBenchName = false;
  }

  promptDeleteBench(): void {
    this.isConfirmingDeleteBench = true;
  }

  cancelDeleteBench(): void {
    this.isConfirmingDeleteBench = false;
  }

  confirmDeleteBench(): void {
    if (!this.activeBench) return;
    const benchId = this.activeBench.id;

    this.apiService.deleteBench(benchId).subscribe({
      next: () => {
        this.benches = this.benches.filter(b => b.id !== benchId);
        this.isConfirmingDeleteBench = false;
        this.isBenchDropdownOpen = false;
        if (this.benches.length > 0) {
          this.selectBench(this.benches[0]);
        } else {
          this.loadBenches();
        }
      },
      error: (err) => console.error('Failed to delete bench', err)
    });
  }

  getFilteredThreads(): Thread[] {
    const query = this.searchQuery.toLowerCase().trim();
    if (!query) return this.threads;
    return this.threads.filter(t => t.title.toLowerCase().includes(query));
  }

  createNewThread(): void {
    if (!this.activeBench) return;
    const title = `Thread ${this.threads.length + 1}`;
    this.apiService.createBenchThread(this.activeBench.id, title).subscribe({
      next: (thread) => {
        this.threads.unshift(thread);
        this.selectThread(thread);
      },
      error: (err) => console.error('Failed to create thread', err)
    });
  }

  selectThread(thread: Thread): void {
    if (!this.activeBench) return;
    this.router.navigate(['/workbench', this.activeBench.id, thread.id]);
  }

  loadThreadContent(thread: Thread): void {
    this.activeThread = thread;
    this.activeThreadMessages = [];
    this.apiService.getMessages(thread.id).subscribe({
      next: (messages) => {
        this.activeThreadMessages = messages;
        this.scrollToBottom();
      },
      error: (err) => console.error('Failed to load messages', err)
    });
    this.loadBenchFiles();
  }

  startEditingThreadInline(thread: Thread, event: Event): void {
    event.stopPropagation();
    this.editingThreadId = thread.id;
    this.editingThreadTitleContent = thread.title;
  }

  saveThreadTitleInline(thread: Thread): void {
    if (this.editingThreadId !== thread.id) return;
    const title = this.editingThreadTitleContent.trim();
    if (!title || title === thread.title) {
      this.editingThreadId = null;
      return;
    }

    this.apiService.updateThread(thread.id, title).subscribe({
      next: (updated) => {
        thread.title = updated.title;
        if (this.activeThread?.id === thread.id) {
          this.activeThread.title = updated.title;
        }
        this.editingThreadId = null;
      },
      error: (err) => {
        console.error('Failed to update thread title', err);
        this.editingThreadId = null;
      }
    });
  }

  cancelEditingThreadInline(): void {
    this.editingThreadId = null;
  }

  promptDeleteThread(threadId: string, event: Event): void {
    event.stopPropagation();
    this.isConfirmingDeleteThreadId = threadId;
  }

  cancelDeleteThread(event?: Event): void {
    if (event) event.stopPropagation();
    this.isConfirmingDeleteThreadId = null;
  }

  confirmDeleteThread(threadId: string, event?: Event): void {
    if (event) event.stopPropagation();

    this.apiService.deleteThread(threadId).subscribe({
      next: () => {
        this.threads = this.threads.filter(t => t.id !== threadId);
        this.isConfirmingDeleteThreadId = null;
        if (this.activeThread?.id === threadId) {
          this.activeThread = null;
          this.activeThreadMessages = [];
          if (this.threads.length > 0 && this.activeBench) {
            this.selectThread(this.threads[0]);
          } else if (this.activeBench) {
            this.router.navigate(['/workbench', this.activeBench.id]);
          }
        }
      },
      error: (err) => console.error('Failed to delete thread', err)
    });
  }

  loadBenchFiles(): void {
    if (!this.activeBench) return;
    this.apiService.listBenchFiles(this.activeBench.id).subscribe({
      next: (res) => {
        this.files = res.files;
        if (this.selectedFile && !this.files.includes(this.selectedFile)) {
          this.selectedFile = null;
          this.selectedFileContent = '';
        }
      },
      error: (err) => console.error('Failed to load bench files', err)
    });
    this.loadBenchMemory();
  }

  loadBenchMemory(): void {
    if (!this.activeBench) return;
    this.apiService.getBenchMemory(this.activeBench.id).subscribe({
      next: (memories) => {
        const working = memories.find(m => m.memory_type === 'working');
        this.benchWorkingMemoryContent = working ? working.content : '';
      },
      error: (err) => console.error('Failed to load bench memory', err)
    });
  }

  saveBenchMemory(): void {
    if (!this.activeBench) return;
    this.isSavingMemory = true;
    this.memorySaveStatus = 'Saving...';
    this.apiService.upsertBenchWorkingMemory(this.activeBench.id, this.benchWorkingMemoryContent).subscribe({
      next: () => {
        this.isSavingMemory = false;
        this.memorySaveStatus = 'Memory saved';
        setTimeout(() => {
          if (this.memorySaveStatus === 'Memory saved') {
            this.memorySaveStatus = '';
          }
        }, 3000);
      },
      error: (err) => {
        this.isSavingMemory = false;
        this.memorySaveStatus = 'Save failed';
        console.error('Failed to save bench memory', err);
      }
    });
  }

  setRightTab(tab: 'files' | 'memory'): void {
    this.activeRightTab = tab;
    if (tab === 'memory') {
      this.loadBenchMemory();
    }
  }

  selectFile(filename: string): void {
    if (!this.activeBench) return;

    if (!filename.endsWith('.txt') && !filename.endsWith('.md')) {
      alert('Only .txt and .md files are supported for editing at this time.');
      return;
    }

    this.apiService.readBenchFile(this.activeBench.id, filename).subscribe({
      next: (res) => {
        this.selectedFile = filename;
        this.selectedFileContent = res.content;
      },
      error: (err) => console.error('Failed to read file', err)
    });
  }

  createNewFile(): void {
    if (!this.activeBench) return;
    const filename = prompt('Enter new filename:');
    if (!filename) return;

    if (!filename.endsWith('.txt') && !filename.endsWith('.md')) {
      alert('Only .txt and .md files are supported at this time.');
      return;
    }

    this.apiService.writeBenchFile(this.activeBench.id, filename, '').subscribe({
      next: () => {
        this.loadBenchFiles();
        this.selectFile(filename);
      },
      error: (err) => console.error('Failed to create file', err)
    });
  }

  saveFile(): void {
    if (!this.activeBench || !this.selectedFile) return;
    this.apiService.writeBenchFile(this.activeBench.id, this.selectedFile, this.selectedFileContent).subscribe({
      next: () => {
        console.log(`Saved ${this.selectedFile}`);
      },
      error: (err) => console.error('Failed to save file', err)
    });
  }

  deleteFile(filename: string, event: Event): void {
    event.stopPropagation();
    if (!this.activeBench) return;

    if (confirm(`Are you sure you want to delete ${filename}?`)) {
      this.apiService.deleteBenchFile(this.activeBench.id, filename).subscribe({
        next: () => {
          if (this.selectedFile === filename) {
            this.selectedFile = null;
            this.selectedFileContent = '';
          }
          this.loadBenchFiles();
        },
        error: (err) => console.error('Failed to delete file', err)
      });
    }
  }

  sendMessage(): void {
    if (!this.newMessageContent.trim() || !this.activeThread || this.isProcessing) {
      return;
    }

    const content = this.newMessageContent.trim();
    const threadId = this.activeThread.id;
    this.newMessageContent = '';
    this.isProcessing = true;

    const tempUserMsg: Message = {
      id: 'temp-' + Date.now(),
      thread_id: threadId,
      role: 'user',
      content,
      created_at: new Date().toISOString()
    };
    this.activeThreadMessages.push(tempUserMsg);
    this.scrollToBottom();

    this.apiService.createMessage(threadId, 'user', content).subscribe({
      next: () => {
        this.apiService.getMessages(threadId).subscribe({
          next: (messages) => {
            this.activeThreadMessages = messages;
            this.isProcessing = false;
            this.scrollToBottom();
            this.loadBenchFiles();
            setTimeout(() => {
              this.messageInput?.nativeElement.focus();
            }, 50);
          },
          error: (err) => {
            console.error('Failed to refresh messages after reply', err);
            this.isProcessing = false;
          }
        });
      },
      error: (err) => {
        console.error('Failed to send message', err);
        this.isProcessing = false;
      }
    });
  }

  scrollToBottom(): void {
    setTimeout(() => {
      if (this.messagesContainer) {
        this.messagesContainer.nativeElement.scrollTop = this.messagesContainer.nativeElement.scrollHeight;
      }
    }, 50);
  }

  startEditingTitle(): void {
    if (!this.activeThread) return;
    this.isEditingTitle = true;
    this.editingTitleContent = this.activeThread.title;
  }

  saveThreadTitle(): void {
    if (!this.activeThread || !this.isEditingTitle) return;
    const title = this.editingTitleContent.trim();
    if (!title || title === this.activeThread.title) {
      this.isEditingTitle = false;
      return;
    }

    this.apiService.updateThread(this.activeThread.id, title).subscribe({
      next: (updated) => {
        if (this.activeThread) {
          this.activeThread.title = updated.title;
        }
        const found = this.threads.find(t => t.id === updated.id);
        if (found) found.title = updated.title;
        this.isEditingTitle = false;
      },
      error: (err) => {
        console.error('Failed to update thread title', err);
        this.isEditingTitle = false;
      }
    });
  }

  cancelEditingTitle(): void {
    this.isEditingTitle = false;
  }
}
