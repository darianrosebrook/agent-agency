/**
 * Rules & Governance API Client
 * 
 * Provides functions for managing CAWS rules, violations, and compliance.
 * 
 * @author @darianrosebrook
 */

import { apiGet, apiPost, apiPatch, apiDelete } from '../utils/api';

/**
 * CAWS Rule
 */
export interface CawsRule {
  id: string;
  name: string;
  description: string;
  rule_type: string;
  severity: string;
  file_patterns: unknown;
  config: unknown;
  constitutional_reference: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

/**
 * CAWS Violation
 */
export interface CawsViolation {
  id: string;
  task_id: string;
  violation_code: string;
  severity: string;
  description: string;
  file_path: string | null;
  line_number: number | null;
  column_number: number | null;
  rule_id: string;
  constitutional_reference: string | null;
  status: string;
  created_at: string;
  resolved_at: string | null;
  metadata: unknown;
}

/**
 * Rule enforcement status
 */
export interface RuleEnforcement {
  rule_id: string;
  is_enforced: boolean;
  enforcement_level: string;
  last_checked: string | null;
}

/**
 * Rule history entry
 */
export interface RuleHistory {
  id: string;
  rule_id: string;
  change_type: string;
  changed_by: string | null;
  old_value: unknown;
  new_value: unknown;
  created_at: string;
}

/**
 * Create rule request
 */
export interface CreateRuleRequest {
  id: string;
  name: string;
  description: string;
  rule_type: string;
  severity: string;
  file_patterns: unknown;
  config: unknown;
  constitutional_reference?: string | null;
  is_active: boolean;
}

/**
 * Update rule request
 */
export interface UpdateRuleRequest {
  name?: string;
  description?: string;
  rule_type?: string;
  severity?: string;
  file_patterns?: unknown;
  config?: unknown;
  constitutional_reference?: string | null;
  is_active?: boolean;
}

/**
 * Validate rule request
 */
export interface ValidateRuleRequest {
  sample_code: string;
  file_path?: string;
}

/**
 * Validate rule response
 */
export interface ValidateRuleResponse {
  matches: boolean;
  violations: Array<{
    line: number;
    column: number;
    message: string;
  }>;
}

/**
 * Rule template
 */
export interface RuleTemplate {
  id: string;
  name: string;
  description: string;
  rule_type: string;
  template_config: unknown;
}

const API_BASE = '/api/proxy/api/v1';

/**
 * List all rules with optional filters
 */
export async function getRules(params?: {
  rule_type?: string;
  is_active?: boolean;
}): Promise<CawsRule[]> {
  const queryParams = new URLSearchParams();
  if (params?.rule_type) queryParams.append('rule_type', params.rule_type);
  if (params?.is_active !== undefined) queryParams.append('is_active', String(params.is_active));
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/rules${queryString ? `?${queryString}` : ''}`;
  return apiGet<CawsRule[]>(url);
}

/**
 * Get a specific rule by ID
 */
export async function getRule(ruleId: string): Promise<CawsRule> {
  return apiGet<CawsRule>(`${API_BASE}/rules/${ruleId}`);
}

/**
 * Create a new rule
 */
export async function createRule(rule: CreateRuleRequest): Promise<CawsRule> {
  return apiPost<CawsRule>(`${API_BASE}/rules`, rule);
}

/**
 * Update an existing rule
 */
export async function updateRule(ruleId: string, updates: UpdateRuleRequest): Promise<CawsRule> {
  return apiPatch<CawsRule>(`${API_BASE}/rules/${ruleId}`, updates);
}

/**
 * Delete a rule
 */
export async function deleteRule(ruleId: string): Promise<void> {
  return apiDelete<void>(`${API_BASE}/rules/${ruleId}`);
}

/**
 * Validate a rule against sample code
 */
export async function validateRule(ruleId: string, request: ValidateRuleRequest): Promise<ValidateRuleResponse> {
  return apiPost<ValidateRuleResponse>(`${API_BASE}/rules/${ruleId}/validate`, request);
}

/**
 * Get rule templates
 */
export async function getRuleTemplates(): Promise<RuleTemplate[]> {
  return apiGet<RuleTemplate[]>(`${API_BASE}/rules/templates`);
}

/**
 * Create a rule from template
 */
export async function createRuleFromTemplate(template: Partial<RuleTemplate>): Promise<RuleTemplate> {
  return apiPost<RuleTemplate>(`${API_BASE}/rules/templates`, template);
}

/**
 * Get rule enforcement status
 */
export async function getRuleEnforcement(ruleId: string): Promise<RuleEnforcement> {
  return apiGet<RuleEnforcement>(`${API_BASE}/rules/${ruleId}/enforcement`);
}

/**
 * Update rule enforcement status
 */
export async function updateRuleEnforcement(ruleId: string, enforcement: Partial<RuleEnforcement>): Promise<RuleEnforcement> {
  return apiPatch<RuleEnforcement>(`${API_BASE}/rules/${ruleId}/enforcement`, enforcement);
}

/**
 * Get rule history
 */
export async function getRuleHistory(ruleId: string): Promise<RuleHistory[]> {
  return apiGet<RuleHistory[]>(`${API_BASE}/rules/${ruleId}/history`);
}

/**
 * List violations with optional filters
 */
export async function getViolations(params?: {
  task_id?: string;
  rule_id?: string;
  status?: string;
}): Promise<CawsViolation[]> {
  const queryParams = new URLSearchParams();
  if (params?.task_id) queryParams.append('task_id', params.task_id);
  if (params?.rule_id) queryParams.append('rule_id', params.rule_id);
  if (params?.status) queryParams.append('status', params.status);
  
  const queryString = queryParams.toString();
  const url = `${API_BASE}/violations${queryString ? `?${queryString}` : ''}`;
  return apiGet<CawsViolation[]>(url);
}

/**
 * Get a specific violation by ID
 */
export async function getViolation(violationId: string): Promise<CawsViolation> {
  return apiGet<CawsViolation>(`${API_BASE}/violations/${violationId}`);
}

/**
 * Update a violation
 */
export async function updateViolation(violationId: string, updates: { status?: string; metadata?: unknown }): Promise<CawsViolation> {
  return apiPatch<CawsViolation>(`${API_BASE}/violations/${violationId}`, updates);
}

/**
 * Resolve a violation
 */
export async function resolveViolation(violationId: string): Promise<CawsViolation> {
  return apiPost<CawsViolation>(`${API_BASE}/violations/${violationId}/resolve`, {});
}

/**
 * Calculate compliance statistics
 */
export interface ComplianceStats {
  total_rules: number;
  active_rules: number;
  total_violations: number;
  open_violations: number;
  resolved_violations: number;
  compliance_score: number;
  violations_by_severity: Record<string, number>;
  violations_by_rule: Array<{
    rule_id: string;
    rule_name: string;
    violation_count: number;
  }>;
}

export async function getComplianceStats(): Promise<ComplianceStats> {
  return apiGet<ComplianceStats>(`${API_BASE}/rules/compliance-stats`);
}

