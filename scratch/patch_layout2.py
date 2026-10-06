with open('aad-fe-container/src/app/components/agent-registry/agent-registry-layout/agent-registry-layout.component.ts', 'r') as f:
    code = f.read()

import re
code = re.sub(r'MatTooltipModule \} from \'@angular/material/button\';', "} from '@angular/material/button';", code)

code = re.sub(r"content: 'Agents act as autonomous nodes(.*?)\'", r"description: 'Agents act as autonomous nodes\1', iconColor: 'text-indigo-500'", code)
code = re.sub(r"content: 'Agents are declarative configurations(.*?)\'", r"description: 'Agents are declarative configurations\1', iconColor: 'text-indigo-500'", code)
code = re.sub(r"content: 'Agents implement Trait Contracts(.*?)\'", r"description: 'Agents implement Trait Contracts\1', iconColor: 'text-indigo-500'", code)

with open('aad-fe-container/src/app/components/agent-registry/agent-registry-layout/agent-registry-layout.component.ts', 'w') as f:
    f.write(code)

