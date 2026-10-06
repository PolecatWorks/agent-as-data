with open('aad-fe-container/src/app/components/agent-registry/agent-empty/agent-empty.component.ts', 'r') as f:
    code = f.read()
code = code.replace("from '../../shared/empty-state/empty-state.component';", "from '../../shared/empty-state/empty-state.component';") # This one is actually correct already since empty-state is at components/shared. Wait, agent-empty is at components/agent-registry/agent-empty/. So it's 2 levels up. ../../shared is correct.
