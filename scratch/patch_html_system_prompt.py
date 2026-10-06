with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.html', 'r') as f:
    code = f.read()

replacement = """
                @if (isEditing) {
                  <textarea rows="12" [(ngModel)]="agentForm.agent_definition" placeholder="You are an autonomous AI agent..."
                            class="font-mono text-sm leading-relaxed text-slate-800 w-full px-3 py-2 bg-white border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"></textarea>
                } @else {
                  <div class="prose prose-slate max-w-none text-sm leading-relaxed text-slate-800 bg-slate-50/55 border border-slate-200/80 rounded-xl p-4 min-h-[150px] overflow-y-auto"
                       [innerHTML]="getRenderedMarkdown(agentForm.agent_definition || '')">
                  </div>
                }
              </div>
"""

import re
code = re.sub(r'                @if \(isEditing\) \{\n                  <textarea rows="12" \[\(ngModel\)\]="agentForm.agent_definition" placeholder="You are an autonomous AI agent..."\n                            class="font-mono text-sm leading-relaxed text-slate-800 w-full px-3 py-2 bg-white border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"></textarea>\n\n\n\n\n\n              </div>', replacement[1:], code)

with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.html', 'w') as f:
    f.write(code)
