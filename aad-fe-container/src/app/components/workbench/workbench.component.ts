import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService, Thread, Message } from '../../services/api.service';

@Component({
  selector: 'app-workbench',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './workbench.component.html',
  styleUrl: './workbench.component.scss'
})
export class WorkbenchComponent implements OnInit {
  threads: Thread[] = [];
  activeThread: Thread | null = null;
  activeThreadMessages: Message[] = [];
  newMessageContent = '';

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.loadThreads();
  }

  loadThreads(): void {
    this.apiService.getThreads().subscribe({
      next: (threads) => {
        this.threads = threads;
        if (this.threads.length > 0 && !this.activeThread) {
          this.selectThread(this.threads[0]);
        }
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
            }
          }
        },
        error: (err) => console.error('Failed to delete thread', err)
      });
    }
  }

  selectThread(thread: Thread): void {
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
}
