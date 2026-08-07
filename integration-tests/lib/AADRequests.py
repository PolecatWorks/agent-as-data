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
