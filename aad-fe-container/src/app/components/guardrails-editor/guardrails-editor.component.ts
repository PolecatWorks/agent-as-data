import { Component, Input, Output, EventEmitter, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSelectModule } from '@angular/material/select';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { GuardrailConfig, ActiveGuardrailItem, ActiveOutputGuardrailItem } from '../../services/api.service';

@Component({
  selector: 'app-guardrails-editor',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatTooltipModule,
    MatSelectModule,
    MatSnackBarModule
  ],
  templateUrl: './guardrails-editor.component.html',
  styleUrls: []
})
export class GuardrailsEditorComponent implements OnInit {
  @Input() guardrails?: GuardrailConfig;
  @Output() guardrailsChange = new EventEmitter<GuardrailConfig>();

  // Available Input Guardrail Catalog for Dropdown selection
  guardrailCatalog = [
    { type: 'prompt_injection', name: 'Prompt Injection Interceptor', tier: 'Tier 1: Deterministic', description: 'Real-time heuristic scanning to intercept injection signatures before reaching LLM' },
    { type: 'pii_regex', name: 'PII Regex Filtering', tier: 'Tier 1: Deterministic', description: 'Regex scanning to block SSNs, emails, credit cards, and secret credentials' },
    { type: 'max_input_tokens', name: 'Max Input Token Limit Cap', tier: 'Tier 1: Deterministic', description: 'Enforce hard upper limit cap on incoming prompt tokens' },
    { type: 'blocked_keywords', name: 'Blocked Input Keyword Blacklist', tier: 'Tier 1: Deterministic', description: 'Reject exact matching phrases or regex keywords in prompts' },
    { type: 'vector_similarity', name: 'Vector Attack Similarity Matcher', tier: 'Tier 2: Vector Matching', description: 'Cosine similarity matching against vector database of known jailbreaks' },
    { type: 'classifier_model', name: 'Specialized Safety Classifier Model', tier: 'Tier 3: Dedicated Classifier', description: 'Route input to Llama Guard, DeBERTa v3, or Perspective API classifier' },
    { type: 'llm_judge', name: 'LLM-as-a-Judge Pre-Evaluator', tier: 'Tier 4: LLM Judge', description: 'Run lightweight fast LLM evaluator against custom policy rules' },
    { type: 'domain_scoping', name: 'System Prompt Domain Scoping', tier: 'Tier 5: System Rules', description: 'Inject allowed topic constraints into system prompt' }
  ];

  // Available Output Guardrail Catalog for Dropdown selection
  outputGuardrailCatalog = [
    { type: 'secret_redaction', name: 'Secret & API Key Redaction', category: 'Data Safety & Privacy', description: 'Automatically scan and mask credentials, RSA private keys, and API tokens in responses' },
    { type: 'pii_ner_redaction', name: 'PII & Sensitive Data Redaction (NER)', category: 'Data Safety & Privacy', description: 'Mask names, emails, phone numbers, SSNs, and credit cards using Named Entity Recognition (Presidio)' },
    { type: 'infra_leakage_filter', name: 'Internal Infra & Network Leakage Filter', category: 'Data Safety & Privacy', description: 'Redact internal stack traces, cluster IPs, private domain names, and database URIs' },
    { type: 'enforce_json_schema', name: 'Strict JSON Schema Contract Enforcement', category: 'Quality & Structure', description: 'Validate output against formal JSON Schema contracts prior to returning payload' },
    { type: 'max_output_tokens', name: 'Max Output Token Generation Limit', category: 'Generation Boundaries', description: 'Hard token limit cap on total output generation per request' },
    { type: 'blocked_output_keywords', name: 'Blocked Output Keyword & Secret Redaction', category: 'Data Safety & Privacy', description: 'Banned terms or secret pattern blacklists redacted from LLM output' },
    { type: 'toxicity_classifier', name: 'Post-Execution Toxicity & Harm Classifier', category: 'Safety & Policy', description: 'Filter hate speech, harassment, or explicit material using Llama Guard or Perspective API' },
    { type: 'brand_competitor_protection', name: 'Competitor & Brand Protection Filter', category: 'Safety & Policy', description: 'Redact or restrict mentions of banned competitor brands or restricted products' },
    { type: 'rag_grounding_hallucination', name: 'RAG Grounding & Hallucination Scoring (NLI)', category: 'Truthfulness & Quality', description: 'Natural Language Inference grounding checks against retrieved context chunks' },
    { type: 'refusal_offtopic_detector', name: 'Refusal & Off-Topic Detector', category: 'Truthfulness & Quality', description: 'Detect hallucinated refusals or domain boundary drift' },
    { type: 'structural_formatting_rules', name: 'Custom Structural & Regex Formatting Rules', category: 'Quality & Structure', description: 'Validate Markdown formatting, custom code block structures, or mandatory URL patterns' }
  ];

  selectedGuardrailTypeToAdd: string = '';
  selectedOutputGuardrailTypeToAdd: string = '';

  constructor(private snackBar: MatSnackBar) {}

  ngOnInit(): void {
    this.ensureGuardrailsStructure();
  }

  ensureGuardrailsStructure(): void {
    if (!this.guardrails) {
      this.guardrails = {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      };
    }
    if (!this.guardrails.input_guardrails) {
      this.guardrails.input_guardrails = { active_guardrails: [] };
    }
    if (!this.guardrails.input_guardrails.active_guardrails) {
      this.guardrails.input_guardrails.active_guardrails = [];
    }
    if (!this.guardrails.output_guardrails) {
      this.guardrails.output_guardrails = { active_guardrails: [] };
    }
    if (!this.guardrails.output_guardrails.active_guardrails) {
      this.guardrails.output_guardrails.active_guardrails = [];
    }
  }

  addGuardrailFromDropdown(): void {
    this.ensureGuardrailsStructure();
    if (!this.selectedGuardrailTypeToAdd) return;
    const catItem = this.guardrailCatalog.find(g => g.type === this.selectedGuardrailTypeToAdd);
    if (!catItem) return;

    const newGuardrail: ActiveGuardrailItem = {
      id: 'tg-' + Date.now(),
      type: catItem.type as any,
      name: catItem.name,
      tier: catItem.tier,
      description: catItem.description,
      config: this.getDefaultConfigForType(catItem.type)
    };

    this.guardrails!.input_guardrails.active_guardrails.push(newGuardrail);
    this.selectedGuardrailTypeToAdd = '';
    this.guardrailsChange.emit(this.guardrails);
    this.snackBar.open(`Added ${catItem.name} guardrail`, 'Close', { duration: 2500 });
  }

  deleteGuardrail(id: string): void {
    if (this.guardrails?.input_guardrails?.active_guardrails) {
      this.guardrails.input_guardrails.active_guardrails = 
        this.guardrails.input_guardrails.active_guardrails.filter(g => g.id !== id);
      this.guardrailsChange.emit(this.guardrails);
      this.snackBar.open('Input guardrail removed', 'Close', { duration: 2000 });
    }
  }

  addOutputGuardrailFromDropdown(): void {
    this.ensureGuardrailsStructure();
    if (!this.selectedOutputGuardrailTypeToAdd) return;
    const catItem = this.outputGuardrailCatalog.find(g => g.type === this.selectedOutputGuardrailTypeToAdd);
    if (!catItem) return;

    const newGuardrail: ActiveOutputGuardrailItem = {
      id: 'og-' + Date.now(),
      type: catItem.type as any,
      name: catItem.name,
      tier: catItem.category,
      description: catItem.description,
      config: this.getDefaultOutputConfigForType(catItem.type)
    };

    this.guardrails!.output_guardrails.active_guardrails.push(newGuardrail);
    this.selectedOutputGuardrailTypeToAdd = '';
    this.guardrailsChange.emit(this.guardrails);
    this.snackBar.open(`Added ${catItem.name} output guardrail`, 'Close', { duration: 2500 });
  }

  deleteOutputGuardrail(id: string): void {
    if (this.guardrails?.output_guardrails?.active_guardrails) {
      this.guardrails.output_guardrails.active_guardrails = 
        this.guardrails.output_guardrails.active_guardrails.filter(g => g.id !== id);
      this.guardrailsChange.emit(this.guardrails);
      this.snackBar.open('Output guardrail removed', 'Close', { duration: 2000 });
    }
  }

  private getDefaultConfigForType(type: string): any {
    switch (type) {
      case 'max_input_tokens':
        return { max_input_tokens: 4096 };
      case 'blocked_keywords':
        return { blocked_input_keywords: ['ignore previous instructions'] };
      case 'vector_similarity':
        return { vector_similarity_threshold: 0.85 };
      case 'classifier_model':
        return { classifier_type: 'llama_guard', toxicity_threshold: 0.7 };
      case 'llm_judge':
        return { judge_model: 'gpt-4o-mini-v2024-07-18', judge_custom_policy_prompt: 'Verify input adheres to safety guidelines.' };
      case 'domain_scoping':
        return { allowed_domain_topics: ['security_auditing', 'code_review'] };
      default:
        return {};
    }
  }

  private getDefaultOutputConfigForType(type: string): any {
    switch (type) {
      case 'secret_redaction':
        return { secret_redaction: true };
      case 'pii_ner_redaction':
        return { pii_ner_entities: ['EMAIL', 'PHONE_NUMBER', 'SSN', 'CREDIT_CARD'] };
      case 'infra_leakage_filter':
        return { infra_leak_types: ['STACK_TRACE', 'INTERNAL_IP', 'DATABASE_URI', 'FILE_PATH'] };
      case 'enforce_json_schema':
        return { enforce_json_schema: true };
      case 'max_output_tokens':
        return { max_output_tokens: 2048 };
      case 'blocked_output_keywords':
        return { blocked_output_keywords: ['AWS_SECRET_ACCESS_KEY', 'PRIVATE KEY'] };
      case 'toxicity_classifier':
        return { classifier_type: 'llama_guard', toxicity_threshold: 0.7 };
      case 'brand_competitor_protection':
        return { banned_competitor_brands: ['CompetitorX', 'RestrictedProductY'] };
      case 'rag_grounding_hallucination':
        return { grounding_min_score: 0.8 };
      case 'refusal_offtopic_detector':
        return { detect_refusal_hallucinations: true };
      case 'structural_formatting_rules':
        return { custom_regex_rules: ['```json[\\s\\S]*```'] };
      default:
        return {};
    }
  }

  addKeywordToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.blocked_input_keywords) g.config.blocked_input_keywords = [];
      g.config.blocked_input_keywords.push(val);
      inputEl.value = '';
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  removeKeywordFromGuardrail(g: any, kw: string): void {
    if (g.config.blocked_input_keywords) {
      g.config.blocked_input_keywords = g.config.blocked_input_keywords.filter((k: string) => k !== kw);
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  addTopicToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.allowed_domain_topics) g.config.allowed_domain_topics = [];
      g.config.allowed_domain_topics.push(val);
      inputEl.value = '';
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  removeTopicFromGuardrail(g: any, topic: string): void {
    if (g.config.allowed_domain_topics) {
      g.config.allowed_domain_topics = g.config.allowed_domain_topics.filter((t: string) => t !== topic);
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  addPiiEntityToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim().toUpperCase();
    if (val) {
      if (!g.config.pii_ner_entities) g.config.pii_ner_entities = [];
      g.config.pii_ner_entities.push(val);
      inputEl.value = '';
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  removePiiEntityFromGuardrail(g: any, entity: string): void {
    if (g.config.pii_ner_entities) {
      g.config.pii_ner_entities = g.config.pii_ner_entities.filter((e: string) => e !== entity);
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  addInfraLeakTypeToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim().toUpperCase();
    if (val) {
      if (!g.config.infra_leak_types) g.config.infra_leak_types = [];
      g.config.infra_leak_types.push(val);
      inputEl.value = '';
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  removeInfraLeakTypeFromGuardrail(g: any, type: string): void {
    if (g.config.infra_leak_types) {
      g.config.infra_leak_types = g.config.infra_leak_types.filter((t: string) => t !== type);
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  addBannedBrandToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.banned_competitor_brands) g.config.banned_competitor_brands = [];
      g.config.banned_competitor_brands.push(val);
      inputEl.value = '';
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  removeBannedBrandFromGuardrail(g: any, brand: string): void {
    if (g.config.banned_competitor_brands) {
      g.config.banned_competitor_brands = g.config.banned_competitor_brands.filter((b: string) => b !== brand);
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  addBlockedOutputKeywordToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.blocked_output_keywords) g.config.blocked_output_keywords = [];
      g.config.blocked_output_keywords.push(val);
      inputEl.value = '';
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  removeBlockedOutputKeywordFromGuardrail(g: any, kw: string): void {
    if (g.config.blocked_output_keywords) {
      g.config.blocked_output_keywords = g.config.blocked_output_keywords.filter((k: string) => k !== kw);
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  addCustomRegexRuleToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.custom_regex_rules) g.config.custom_regex_rules = [];
      g.config.custom_regex_rules.push(val);
      inputEl.value = '';
      this.guardrailsChange.emit(this.guardrails);
    }
  }

  removeCustomRegexRuleFromGuardrail(g: any, rule: string): void {
    if (g.config.custom_regex_rules) {
      g.config.custom_regex_rules = g.config.custom_regex_rules.filter((r: string) => r !== rule);
      this.guardrailsChange.emit(this.guardrails);
    }
  }
}
