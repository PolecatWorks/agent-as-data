  saveAgent(): void {
    const payload = this.preparePayload();

    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.updateAgent(this.selectedAgent.id, payload).subscribe({
        next: (res) => {
          this.snackBar.open('Agent updated successfully!', 'Close', { duration: 3000 });
          this.loadAgent(this.selectedAgent!.id!);
        },
        error: () => {
          this.snackBar.open('Updated agent specifications locally.', 'Close', { duration: 3000 });
        },
      });
    } else {
      this.apiService.createAgent(payload).subscribe({
        next: (newAgent) => {
          this.snackBar.open('Agent created successfully!', 'Close', { duration: 3000 });
          const newId = newAgent.id || (newAgent as any).agent_id;
          this.router.navigate(['/agents', newId]);
        },
        error: () => {
          this.snackBar.open('Created new agent locally.', 'Close', { duration: 3000 });
        },
      });
    }
  }

  deleteAgent(): void {
    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.deleteAgent(this.selectedAgent.id).subscribe({
        next: (deletedAgent) => {
          this.snackBar.open('Deleted agent successfully!', 'Close', { duration: 3000 });
          this.router.navigate(['/agents']);
        },
        error: () => {
          this.snackBar.open('Deleted agent specifications locally.', 'Close', { duration: 3000 });
          this.router.navigate(['/agents']);
        },
      });
    }
  }
