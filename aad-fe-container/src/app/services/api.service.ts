import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, of } from 'rxjs';
import { map } from 'rxjs/operators';

export type InputGuardrailType = 
  | 'prompt_injection'
  | 'pii_regex'
  | 'max_input_tokens'
  | 'blocked_keywords'
  | 'vector_similarity'
  | 'classifier_model'
  | 'llm_judge'
  | 'domain_scoping';

export interface ActiveGuardrailItem {
  id: string;
  type: InputGuardrailType;
  name: string;
  tier: string;
  description: string;
  config: {
    max_input_tokens?: number;
    blocked_input_keywords?: string[];
    vector_similarity_threshold?: number;
    classifier_type?: 'llama_guard' | 'deberta_v3' | 'perspective_api';
    toxicity_threshold?: number;
    judge_model?: string;
    judge_custom_policy_prompt?: string;
    allowed_domain_topics?: string[];
  };
}

export interface InputGuardrails {
  active_guardrails: ActiveGuardrailItem[];
}

export type OutputGuardrailType =
  | 'secret_redaction'
  | 'pii_ner_redaction'
  | 'infra_leakage_filter'
  | 'enforce_json_schema'
  | 'max_output_tokens'
  | 'blocked_output_keywords'
  | 'toxicity_classifier'
  | 'brand_competitor_protection'
  | 'rag_grounding_hallucination'
  | 'refusal_offtopic_detector'
  | 'structural_formatting_rules';

export interface ActiveOutputGuardrailItem {
  id: string;
  type: OutputGuardrailType;
  name: string;
  tier: string;
  description: string;
  config: {
    secret_redaction?: boolean;
    pii_ner_entities?: string[];
    infra_leak_types?: string[];
    enforce_json_schema?: boolean;
    max_output_tokens?: number;
    blocked_output_keywords?: string[];
    classifier_type?: 'llama_guard' | 'perspective_api';
    toxicity_threshold?: number;
    banned_competitor_brands?: string[];
    grounding_min_score?: number;
    detect_refusal_hallucinations?: boolean;
    custom_regex_rules?: string[];
  };
}

export interface OutputGuardrails {
  active_guardrails: ActiveOutputGuardrailItem[];
}


export interface GuardrailConfig {
  input_guardrails: InputGuardrails;
  output_guardrails: OutputGuardrails;
}

export interface Thread {
  id: string;
  owner_id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface Message {
  id: string;
  thread_id: string;
  role: string;
  content: string;
  created_at: string;
}


export interface Agent {
  id: string;
  name: string;
  description: string;
  tags: string[];
  implements_traits: string[];
  attached_tools?: string[];
  attached_agents?: string[];
  attached_skills?: string[];
  current_version: string;
  owner_id: string;
  judge_threshold: number;
  read_groups?: string[];
  write_groups?: string[];
  execute_groups?: string[];
  agent_definition?: string;
  model?: string;
  input_guardrails_enums?: InputGuardrailType[];
  output_guardrails_enums?: OutputGuardrailType[];
  input_guardrails?: string[];
  output_guardrails?: string[];
  guardrails?: GuardrailConfig;
  guardrail_config?: any;
}




export interface TraitContract {
  id: string;
  name: string;
  owner_id: string;
  description: string;
  version: string;
  capability_requirements: string[];
  behavioral_invariants: string[];
  evaluation_criteria: string[];
  tags: string[];
  input_guardrails_enums?: InputGuardrailType[];
  output_guardrails_enums?: OutputGuardrailType[];
  guardrails?: GuardrailConfig;
}



export interface Skill {
  id?: string;
  name: string;
  description: string;
  definition: string;
  tags: string[];
  implements_traits?: string[];
  current_version: string;
  attached_skills?: string[];
  attached_tools?: string[];
  owner_id: string;
  input_schema?: any;
  output_schema?: any;
  implementation?: any;
}


@Injectable({
  providedIn: 'root'
})
export class ApiService {
  private baseUrl = '/api/v1';

  constructor(private http: HttpClient) {}

  // Agent Registry APIs
  getAgents(): Observable<Agent[]> {
    return this.http.post<any[]>(`${this.baseUrl}/agents/search`, { query: '', limit: 50 }).pipe(
      map(agents => agents.map(agent => ({
        ...agent,
        id: agent.id || agent.agent_id
      })))
    );
  }

  createAgent(agent: Agent): Observable<Agent> {
    return this.http.post<Agent>(`${this.baseUrl}/agents`, agent);
  }

  updateAgent(id: string, agent: Agent): Observable<Agent> {
    return this.http.put<Agent>(`${this.baseUrl}/agents/${id}`, agent);
  }

  getAgent(id: string): Observable<Agent> {
    return this.http.get<Agent>(`${this.baseUrl}/agents/${id}`);
  }

  deleteAgent(id: string): Observable<Agent> {
    return this.http.delete<Agent>(`${this.baseUrl}/agents/${id}`);
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
  // TODO: replace ANONYMOUS_OWNER_ID with the authenticated user's UUID once auth is implemented
  private readonly ANONYMOUS_OWNER_ID = '00000000-0000-0000-0000-000000000000';

  registerTool(serverName: string, transportType: string, endpointConfig: any, ownerId?: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/tools/register`, {
      server_name: serverName,
      transport_type: transportType,
      endpoint_config: endpointConfig,
      owner_id: ownerId || this.ANONYMOUS_OWNER_ID
    });
  }

  getTools(): Observable<any[]> {
    return this.http.get<any[]>(`${this.baseUrl}/agents/tools`);
  }

  deleteTool(id: string): Observable<any> {
    return this.http.delete(`${this.baseUrl}/agents/tools/${id}`);
  }

  // Refactoring APIs
  analyzeRefactor(): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/refactor/analyze`, {});
  }

  // Workbench Threads APIs
  getThreads(ownerId?: string): Observable<Thread[]> {
    return this.http.post<Thread[]>(`${this.baseUrl}/threads`, {
      owner_id: ownerId || this.ANONYMOUS_OWNER_ID,
      pagination: { page: 0, size: 50 }
    });
  }

  createThread(title: string, ownerId?: string): Observable<Thread> {
    return this.http.post<Thread>(`${this.baseUrl}/threads/create`, {
      title,
      owner_id: ownerId || this.ANONYMOUS_OWNER_ID
    });
  }

  updateThread(id: string, title: string): Observable<Thread> {
    return this.http.put<Thread>(`${this.baseUrl}/threads/${id}`, {
      title
    });
  }

  deleteThread(id: string): Observable<any> {
    return this.http.delete(`${this.baseUrl}/threads/${id}`);
  }

  getMessages(threadId: string): Observable<Message[]> {
    return this.http.get<Message[]>(`${this.baseUrl}/threads/${threadId}/messages`);
  }

  createMessage(threadId: string, role: string, content: string): Observable<Message> {
    return this.http.post<Message>(`${this.baseUrl}/threads/${threadId}/messages`, {
      role,
      content
    });
  }

  // Trait Contract APIs
  getTraits(): Observable<ListPages> {
    return this.http.get<ListPages>(`${this.baseUrl}/traits`);
  }

  getTrait(id: string): Observable<TraitContract> {
    return this.http.get<TraitContract>(`${this.baseUrl}/traits/${id}`);
  }

  createTrait(trait: Partial<TraitContract>): Observable<TraitContract> {
    return this.http.post<TraitContract>(`${this.baseUrl}/traits`, trait);
  }

  updateTrait(id: string, trait: Partial<TraitContract>): Observable<TraitContract> {
    return this.http.put<TraitContract>(`${this.baseUrl}/traits/${id}`, trait);
  }

  deleteTrait(id: string): Observable<any> {
    return this.http.delete(`${this.baseUrl}/traits/${id}`);
  }

  // Skill Registry APIs
  getSkills(): Observable<Skill[]> {
    return this.http.get<Skill[]>(`${this.baseUrl}/skills`);
  }

  getSkill(id: string): Observable<Skill> {
    return this.http.get<Skill>(`${this.baseUrl}/skills/${id}`);
  }

  createSkill(skill: Partial<Skill>): Observable<Skill> {
    return this.http.post<Skill>(`${this.baseUrl}/skills`, skill);
  }

  updateSkill(id: string, skill: Partial<Skill>): Observable<Skill> {
    return this.http.put<Skill>(`${this.baseUrl}/skills/${id}`, skill);
  }

  deleteSkill(id: string): Observable<any> {
    return this.http.delete(`${this.baseUrl}/skills/${id}`);
  }

  promoteSkill(id: string): Observable<Agent> {
    return this.http.post<Agent>(`${this.baseUrl}/skills/${id}/promote`, {});
  }

  demoteAgent(id: string): Observable<any> {
    return this.http.post<any>(`${this.baseUrl}/agents/${id}/demote`, {});
  }
}

export interface ListPages {
  ids: string[];
  pagination: {
    page?: number;
    size?: number;
  };
}
