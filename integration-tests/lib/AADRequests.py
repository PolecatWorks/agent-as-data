import os
import requests
import time

class AADRequests:
    """Robot Framework Library for Agent-As-Data REST & MCP HTTP Requests."""
    
    ROBOT_LIBRARY_SCOPE = 'GLOBAL'
    
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url.rstrip('/')
        
    def set_backend_url(self, url):
        self.base_url = url.rstrip('/')

    def check_health(self):
        try:
            resp = requests.get(f"{self.base_url}/health", timeout=2)
            return resp.status_code == 200
        except Exception:
            return False

    def create_agent(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents", json=payload, timeout=10)
        resp.raise_for_status()
        return resp.json()

    def update_agent(self, agent_id, payload):
        resp = requests.put(f"{self.base_url}/api/v1/agents/{agent_id}", json=payload, timeout=10)
        resp.raise_for_status()
        return resp.json()

    def test_agent(self, agent_id, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/{agent_id}/test", json=payload, timeout=120)
        resp.raise_for_status()
        return resp.json()

    def list_traits(self):
        resp = requests.get(f"{self.base_url}/api/v1/traits", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def get_trait(self, trait_id):
        resp = requests.get(f"{self.base_url}/api/v1/traits/{trait_id}", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def create_trait(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/traits", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def update_trait(self, trait_id, payload):
        resp = requests.put(f"{self.base_url}/api/v1/traits/{trait_id}", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def delete_trait(self, trait_id):
        resp = requests.delete(f"{self.base_url}/api/v1/traits/{trait_id}", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def delete_agent(self, agent_id, hard=None):
        params = {}
        if hard is not None:
            params['hard'] = str(hard).lower()
        resp = requests.delete(f"{self.base_url}/api/v1/agents/{agent_id}", params=params, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def execute_agent(self, agent_id, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/{agent_id}/execute", json=payload, timeout=180)
        resp.raise_for_status()
        return resp.json()

    def create_skill(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/skills", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def register_tool(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/tools/register", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def ingest_knowledge(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/knowledge", json=payload, timeout=15)
        resp.raise_for_status()
        return resp.json()

    def search_knowledge(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/knowledge/search", json=payload, timeout=15)
        resp.raise_for_status()
        return resp.json()

    def traverse_graph(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/knowledge/graph/traverse", json=payload, timeout=15)
        resp.raise_for_status()
        return resp.json()

    def compile_agent(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/compile", json=payload, timeout=15)
        resp.raise_for_status()
        return resp.json()

    def search_and_execute(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/search-and-execute", json=payload, timeout=180)
        resp.raise_for_status()
        return resp.json()

    def search_agents(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/search", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def promote_skill(self, skill_id):
        resp = requests.post(f"{self.base_url}/api/v1/skills/{skill_id}/promote", timeout=2)
        resp.raise_for_status()
        return resp.json()

    def delete_skill(self, skill_id):
        resp = requests.delete(f"{self.base_url}/api/v1/skills/{skill_id}", timeout=2)
        resp.raise_for_status()
        return resp.json()

    def analyze_refactor(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/refactor/analyze", json=payload, timeout=2)
        resp.raise_for_status()
        return resp.json()

    def verify_contract(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/verify-contract", json=payload, timeout=2)
        resp.raise_for_status()
        return resp.json()

    # Benches & Threads APIs
    def list_benches(self):
        resp = requests.get(f"{self.base_url}/api/v1/benches", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def create_bench(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/benches/create", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def get_bench(self, bench_id):
        resp = requests.get(f"{self.base_url}/api/v1/benches/{bench_id}", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def update_bench(self, bench_id, payload):
        resp = requests.put(f"{self.base_url}/api/v1/benches/{bench_id}", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def delete_bench(self, bench_id):
        resp = requests.delete(f"{self.base_url}/api/v1/benches/{bench_id}", timeout=5)
        resp.raise_for_status()
        return resp.status_code == 204

    def list_bench_threads(self, bench_id):
        resp = requests.get(f"{self.base_url}/api/v1/benches/{bench_id}/threads", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def create_bench_thread(self, bench_id, payload):
        resp = requests.post(f"{self.base_url}/api/v1/benches/{bench_id}/threads", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def delete_thread(self, thread_id):
        resp = requests.delete(f"{self.base_url}/api/v1/threads/{thread_id}", timeout=5)
        resp.raise_for_status()
        return resp.status_code == 204

    def list_bench_files(self, bench_id, dir_path=""):
        resp = requests.post(f"{self.base_url}/api/v1/benches/{bench_id}/fs/list", json={"dir_path": dir_path}, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def write_bench_file(self, bench_id, filepath, content):
        resp = requests.post(f"{self.base_url}/api/v1/benches/{bench_id}/fs/write", json={"filepath": filepath, "content": content}, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def read_bench_file(self, bench_id, filepath):
        resp = requests.get(f"{self.base_url}/api/v1/benches/{bench_id}/fs/read/{filepath}", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def delete_bench_file(self, bench_id, filepath):
        resp = requests.post(f"{self.base_url}/api/v1/benches/{bench_id}/fs/delete", json={"filepath": filepath}, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def get_bench_memory(self, bench_id):
        resp = requests.get(f"{self.base_url}/api/v1/benches/{bench_id}/memory", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def upsert_bench_memory(self, bench_id, payload):
        resp = requests.put(f"{self.base_url}/api/v1/benches/{bench_id}/memory", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def append_bench_decision(self, bench_id, payload):
        resp = requests.post(f"{self.base_url}/api/v1/benches/{bench_id}/memory/decision", json=payload, timeout=5)
        resp.raise_for_status()
        return resp.json()

    def create_thread_message(self, thread_id, payload):
        resp = requests.post(f"{self.base_url}/api/v1/threads/{thread_id}/messages", json=payload, timeout=180)
        resp.raise_for_status()
        return resp.json()

    def get_thread_messages(self, thread_id):
        resp = requests.get(f"{self.base_url}/api/v1/threads/{thread_id}/messages", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def get_active_thread_run(self, thread_id):
        resp = requests.get(f"{self.base_url}/api/v1/threads/{thread_id}/runs/active", timeout=5)
        if resp.status_code == 204:
            return None
        resp.raise_for_status()
        return resp.json()

    def cancel_active_thread_run(self, thread_id):
        resp = requests.post(f"{self.base_url}/api/v1/threads/{thread_id}/runs/active/cancel", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def list_thread_runs(self, thread_id):
        resp = requests.get(f"{self.base_url}/api/v1/threads/{thread_id}/runs", timeout=5)
        resp.raise_for_status()
        return resp.json()

    def wait_for_assistant_message(self, thread_id, timeout=30):
        start = time.time()
        while time.time() - start < timeout:
            msgs = self.get_thread_messages(thread_id)
            if any(m.get("role") in ("assistant", "system") for m in msgs):
                return msgs
            time.sleep(0.5)
        raise TimeoutError(f"Assistant message was not generated within {timeout} seconds")

    def delete_knowledge(self, node_id):
        resp = requests.delete(f"{self.base_url}/api/v1/knowledge/{node_id}", timeout=5)
        resp.raise_for_status()
        return resp.status_code == 204

    def _get_var(self, name, default=None):
        """Helper to get a variable from Robot Framework BuiltIn context or os.environ."""
        try:
            from robot.libraries.BuiltIn import BuiltIn
            val = BuiltIn().get_variable_value(f"${{{name}}}")
            if val is not None and str(val).strip() != "":
                return str(val).strip()
        except Exception:
            pass
        return os.environ.get(name, default)

    def resolve_hams_endpoint(self, service="backend"):
        """
        Resolves the base URL for the HaMS telemetry endpoint of the requested service.
        Hierarchy:
          1. Direct URL override (HAMS_BE_URL / HAMS_MCP_URL)
          2. Pod IP direct addressing (BE_POD_IP / MCP_POD_IP) with port 8079
          3. Local development loopback fallback (:8079 for backend, :8078 for mcp)
        """
        s = service.lower().strip()
        if s in ("backend", "be", "aad-be-container", "agent-as-data"):
            override = self._get_var("HAMS_BE_URL")
            if override:
                return override.rstrip("/")
            pod_ip = self._get_var("BE_POD_IP")
            if pod_ip:
                return f"http://{pod_ip}:8079"
            return "http://localhost:8079"
        elif s in ("mcp", "aad-mcp-container", "agent-as-data-mcp"):
            override = self._get_var("HAMS_MCP_URL")
            if override:
                return override.rstrip("/")
            pod_ip = self._get_var("MCP_POD_IP")
            if pod_ip:
                return f"http://{pod_ip}:8079"
            return "http://localhost:8078"
        else:
            raise ValueError(f"Unknown service '{service}'. Expected 'backend' or 'mcp'.")

    def get_hams_metrics(self, service="backend", timeout=5):
        """
        Fetches the Prometheus exposition text from the resolved HaMS telemetry endpoint.
        """
        endpoint = f"{self.resolve_hams_endpoint(service)}/hams/metrics"
        resp = requests.get(endpoint, timeout=timeout)
        resp.raise_for_status()
        return resp.text

    def get_hams_health(self, service="backend", endpoint="ready", timeout=5):
        """
        Checks health status (/hams/alive or /hams/ready) on the resolved HaMS endpoint.
        """
        target = endpoint.lstrip("/")
        url = f"{self.resolve_hams_endpoint(service)}/hams/{target}"
        try:
            resp = requests.get(url, timeout=timeout)
            return resp.status_code == 200
        except Exception:
            return False

