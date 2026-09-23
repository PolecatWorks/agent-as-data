# Workbench Multi-Turn Journeys PRD

## Overview
This document outlines the requirements and architectural considerations for enabling complex, multi-turn conversational interactions within the Agent-As-Data Workbench. Specifically, it details three foundational user journeys designed to test and demonstrate the platform's capabilities in code generation, file manipulation, and secure code execution.

## Core Objectives
1. **Demonstrate Statefulness:** Validate that the agent can maintain context over multiple turns within a specific Workbench Thread.
2. **Exercise File Tools:** Ensure the agent can accurately read, write, and modify files within the isolated Bench filesystem (`/tmp/workspace/benches/<bench_id>`).
3. **Secure Execution:** Introduce a secure runtime environment to allow the agent to execute dynamically generated code without risking the host or backend stability.

## Architectural Requirement: Code Execution MCP Server
To support journeys that require running code (Journeys 1 & 3), we require a secure execution sandbox. Running arbitrary LLM-generated code directly on the backend container introduces unacceptable security risks.

**Solution:** Implement a dedicated **Code Execution MCP Server**.
*   **Separation of Concerns:** This will be a separate, highly restricted Docker container (or service) responsible solely for executing code.
*   **MCP Integration:** It will expose an MCP (Model Context Protocol) tool, for example, `mcp:code_runner:execute_python`.
*   **Shared Volume/Sync:** The runner will have access to the specific Bench's filesystem (e.g., via volume mounts mapped to `/tmp/workspace/benches/<bench_id>`) to execute files created by the agent.
*   **Security Posture:** The runner container must execute processes as a non-root user, have strict CPU/Memory resource limits (cgroups), and ideally have its network access disabled or heavily firewalled to prevent data exfiltration or external attacks.

## Target User Journeys

### Journey 1: Creating a Simple Python Script (Code Generation & Execution)
This journey validates the agent's ability to write code to the filesystem and securely execute it via the MCP Runner.

*   **Turn 1: User Request**
    *   *Prompt:* "Can you write a simple Python script called `hello.py` that prints 'Hello Workbench'?"
    *   *Agent Action:* The agent generates the Python code and uses the native workspace file tools to create `/tmp/workspace/benches/<bench_id>/hello.py`.
    *   *Agent Response:* Confirms the file has been created.
*   **Turn 2: Execution Request**
    *   *Prompt:* "Did you create it? Can you run it and tell me the output?"
    *   *Agent Action:* The agent utilizes the new `mcp:code_runner:execute_python` tool, pointing it to `hello.py`.
    *   *Agent Response:* Returns the standard output ("Hello Workbench") back to the user in the chat.

### Journey 2: Brainstorming & Documenting (Knowledge & Markdown)
This journey tests conversational brainstorming, context retention, and basic file writing without the need for code execution.

*   **Turn 1: Ideation**
    *   *Prompt:* "Let's brainstorm some ideas for a new 'Task Management' feature. Give me 3 bullet points."
    *   *Agent Action:* Generates ideas based on prompt (and potentially contextual knowledge).
    *   *Agent Response:* Provides 3 distinct bullet points in the chat.
*   **Turn 2: Documentation**
    *   *Prompt:* "I like the second one. Can you create a markdown file named `feature_idea.md` and document that specific point in detail?"
    *   *Agent Action:* Uses context to identify the "second point", expands upon it, and uses workspace file tools to write it to `feature_idea.md`.
    *   *Agent Response:* Confirms the documentation is saved.

### Journey 3: Debugging an Error (File Editing & Resolution)
This journey exercises the agent's ability to read existing files, reason about code defects, overwrite files with fixes, and verify the fix via execution.

*   **Turn 1: Setup Buggy File**
    *   *Prompt:* "I have a file called `math.py` with a bug. Create it with this code: `def add(a, b): return a - b`."
    *   *Agent Action:* Uses workspace file tools to write the buggy code to `math.py`.
    *   *Agent Response:* Confirms creation.
*   **Turn 2: Identify and Fix Bug**
    *   *Prompt:* "There's a bug in `math.py` when I try to add numbers. Can you fix it and test it?"
    *   *Agent Action:*
        1. Reads `math.py` to identify the bug (`-` instead of `+`).
        2. Writes the corrected code back to `math.py`.
        3. (Optional but desired) Creates a temporary test script or uses the MCP Code Runner directly to evaluate `add(2, 2)` to verify the fix.
    *   *Agent Response:* Explains the bug found, confirms it was fixed, and reports the successful test result.

## Future Considerations
*   **Multi-Language Support:** The initial MCP Runner may focus on Python, but should be extensible to Node.js, bash, and other runtimes.
*   **Timeouts & Halts:** The MCP Runner must enforce strict execution timeouts to prevent infinite loops (e.g., `while True: pass`) from locking up runner resources.
