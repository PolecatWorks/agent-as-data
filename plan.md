1. **Define HTML Tooltip Function**
   - Create a helper method in `knowledge-explorer.component.ts` called `createHtmlTooltip(name: string, description: string | undefined): HTMLElement` (or return an HTML string if `vis-network` supports it).
   - This function will format the `name` and `description` (or fallback "not description" if absent) into an HTML layout.

2. **Update Entity Node Processors to Include `title`**
   - **Process Agents:** Pass `title: this.createHtmlTooltip(agent.name, agent.description)` in `newNodes.push`.
   - **Process Skills:** Pass `title: this.createHtmlTooltip(skill.name, skill.description)` in `newNodes.push`.
   - **Process Tools:** Pass `title: this.createHtmlTooltip(toolName, tool.description)` in `newNodes.push`.
   - **Process Traits:** Pass `title: this.createHtmlTooltip(trait.name, trait.description)` in `newNodes.push`.
   - **Process Knowledge Nodes:** Pass `title: this.createHtmlTooltip(title, node.description || node.content)` in `newNodes.push`.
   - Do the same for inline-created traits and concepts within tuples mapping logic.

3. **Verify Vis-Network HTML Rendering Support**
   - Ensure `vis-network` renders HTML tooltips correctly by passing raw HTML string or a DOM element.

4. **Complete Pre-Commit Steps**
   - Ensure proper testing, verification, review, and reflection are done by calling `pre_commit_instructions` and adhering to it.

5. **Submit Change**
