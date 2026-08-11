import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, of } from 'rxjs';

export interface Agent {
  id: string;
  name: string;
  description: string;
  tags: string[];
  implements_traits: string[];
  current_version: number;
  owner_id: string;
  judge_threshold: number;
  read_groups?: string[];
  write_groups?: string[];
  execute_groups?: string[];
  agent_definition?: string;
  model?: any;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  tags: string[];
  current_version: number;
  owner_id: string;
}

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  private baseUrl = '/api/v1';

  constructor(private http: HttpClient) {}

  // Agent Registry APIs
  getAgents(): Observable<Agent[]> {
    return this.http.post<Agent[]>(`${this.baseUrl}/agents/search`, { query: '', limit: 50 });
  }

  createAgent(agent: Partial<Agent>): Observable<Agent> {
    return this.http.post<Agent>(`${this.baseUrl}/agents`, {
      name: agent.name || 'New Agent',
      description: agent.description || '',
      tags: agent.tags || [],
      implements_traits: agent.implements_traits || [],
      owner_id: agent.owner_id || '00000000-0000-0000-0000-000000000000',
      agent_definition: agent.agent_definition || 'You are a helpful AI assistant.',
      read_groups: agent.read_groups || [],
      write_groups: agent.write_groups || [],
      execute_groups: agent.execute_groups || [],
      judge_threshold: agent.judge_threshold || 0.8
    });
  }

  updateAgent(id: string, agent: Partial<Agent>): Observable<Agent> {
    return this.http.put<Agent>(`${this.baseUrl}/agents/${id}`, agent);
  }

  verifyContract(targetAgentId: string, traitName: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/verify-contract`, {
      target_agent_id: targetAgentId,
      trait_name: traitName
    });
  }

  compileNetwork(rootAgentId: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/compile`, { root_agent_id: rootAgentId });
  }

  // Execution APIs
  executeAgent(agentId: string, prompt: string, webhookUrl?: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/${agentId}/execute`, { prompt, webhook_url: webhookUrl });
  }

  searchAndExecute(prompt: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/search-and-execute`, { prompt });
  }

  // Knowledge APIs
  searchKnowledge(query: string): Observable<any[]> {
    return this.http.post<any[]>(`${this.baseUrl}/knowledge/search`, { query, limit: 10 });
  }

  ingestKnowledge(topic: string, title: string, content: string, tuples?: any[]): Observable<any> {
    return this.http.post(`${this.baseUrl}/knowledge`, { topic, title, content, tuples });
  }

  traverseGraph(subject: string): Observable<any[]> {
    return this.http.post<any[]>(`${this.baseUrl}/knowledge/graph/traverse`, { subject });
  }

  // MCP APIs
  registerMcpServer(serverName: string, transportType: string, endpointConfig: any): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/mcp/register`, {
      server_name: serverName,
      transport_type: transportType,
      endpoint_config: endpointConfig
    });
  }

  // Refactoring APIs
  analyzeRefactor(): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/refactor/analyze`, {});
  }
}
