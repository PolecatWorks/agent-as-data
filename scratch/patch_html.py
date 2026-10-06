with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.html', 'r') as f:
    lines = f.readlines()

# line 4 is index 3
lines[3] = "\n"

# The @else block starts around line 220. We find it and remove up to }
start_idx = -1
for i, line in enumerate(lines):
    if "} @else {" in line:
        start_idx = i
        break

if start_idx != -1:
    end_idx = start_idx
    for i in range(start_idx + 1, len(lines)):
        if "}" in lines[i]:
            end_idx = i
            break
    
    for i in range(start_idx, end_idx + 1):
        lines[i] = "\n"

# Remove any extra </div> that are dangling at the end
# The file originally had 2 </div> for the layout. Let's make sure only 1 </div> is left for the `<div class="h-full...">`
with open('aad-fe-container/src/app/components/agent-registry/agent-detail.component.html', 'w') as f:
    f.writelines(lines)
