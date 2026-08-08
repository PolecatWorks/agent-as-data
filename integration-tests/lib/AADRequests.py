import os
import requests

class AADRequests:
    """Robot Framework Library for Agent-As-Data REST & MCP HTTP Requests."""
    
    ROBOT_LIBRARY_SCOPE = 'GLOBAL'
    
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url.rstrip('/')
        
    def set_backend_url(self, url):
        self.base_url = url.rstrip('/')

    def check_health(self):
        resp = requests.get(f"{self.base_url}/health")
        return resp.status_code == 200

    def create_agent(self, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents", json=payload)
        resp.raise_for_status()
        return resp.json()

    def update_agent(self, agent_id, payload):
        resp = requests.put(f"{self.base_url}/api/v1/agents/{agent_id}", json=payload)
        resp.raise_for_status()
        return resp.json()

    def test_agent(self, agent_id, payload):
        resp = requests.post(f"{self.base_url}/api/v1/agents/{agent_id}/test", json=payload)
        resp.raise_for_status()
        return resp.json()
