with open('aad-fe-container/src/app/components/agent-registry/agent-registry-layout/agent-registry-layout.component.ts', 'r') as f:
    code = f.read()

# Fix 1: Add MatTooltipModule
code = code.replace("import { MatButtonModule } from '@angular/material/button';", "import { MatButtonModule } from '@angular/material/button';\nimport { MatTooltipModule } from '@angular/material/tooltip';")
code = code.replace("MatButtonModule", "MatButtonModule,\n    MatTooltipModule")

# Fix 2: Remove `type` from conceptGuideMappings
code = code.replace("type: 'info', ", "")
code = code.replace("type: 'settings', ", "")
code = code.replace("type: 'security', ", "")

with open('aad-fe-container/src/app/components/agent-registry/agent-registry-layout/agent-registry-layout.component.ts', 'w') as f:
    f.write(code)

