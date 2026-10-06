with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.ts', 'r') as f:
    code = f.read()

# Replace the extra closing brace and anything else appended by accident
code = code[:code.rfind("}\n  enableEdit(): void {")] + """
  enableEdit(): void {
    this.isEditing = true;
    this.router.navigate([], {
      relativeTo: this.route,
      queryParams: { edit: 'true' },
      queryParamsHandling: 'merge'
    });
  }
}
"""

with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.ts', 'w') as f:
    f.write(code)

