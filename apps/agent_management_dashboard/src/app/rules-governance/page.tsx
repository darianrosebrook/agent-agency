"use client";

import { useState, useEffect } from "react";
import styles from "./page.module.scss";
import {
  getRules,
  getViolations,
  getComplianceStats,
  type CawsRule,
  type CawsViolation,
  type ComplianceStats,
} from "../../lib/api/rules";
import { ErrorDisplay } from "../../components/ErrorDisplay";
import { BentoPanel } from "../../components/compounds/BentoPanel";

/**
 * Rules & Governance Page
 * 
 * Management and oversight of coding rules, governance policies,
 * quality gates, and compliance standards.
 * 
 * @author @darianrosebrook
 */
export default function RulesGovernancePage() {
  const [rules, setRules] = useState<CawsRule[]>([]);
  const [violations, setViolations] = useState<CawsViolation[]>([]);
  const [compliance, setCompliance] = useState<ComplianceStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [filter, setFilter] = useState<{
    ruleType?: string;
    isActive?: boolean;
    violationStatus?: string;
  }>({});
  const [selectedRule, setSelectedRule] = useState<CawsRule | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        const [rulesData, violationsData, complianceData] = await Promise.all([
          getRules({ rule_type: filter.ruleType, is_active: filter.isActive }),
          getViolations({ status: filter.violationStatus }),
          getComplianceStats(),
        ]);

        setRules(Array.isArray(rulesData) ? rulesData : []);
        setViolations(Array.isArray(violationsData) ? violationsData : []);
        setCompliance(complianceData);
      } catch (err) {
        setError(err instanceof Error ? err : new Error("Failed to load rules and governance data"));
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, [filter.ruleType, filter.isActive, filter.violationStatus]);

  if (isLoading) {
    return (
      <div className={styles.rulesGovernancePage}>
        <div className={styles.container}>
          <div className={styles.header}>
            <h1 className={styles.headerTitle}>Rules & Governance</h1>
            <p className={styles.headerDescription}>Loading rules and compliance data...</p>
          </div>
          <div className={styles.loadingState}>
            <div className={styles.spinner}></div>
            <p>Loading data...</p>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.rulesGovernancePage}>
        <div className={styles.container}>
          <div className={styles.header}>
            <h1 className={styles.headerTitle}>Rules & Governance</h1>
          </div>
          <ErrorDisplay
            error={error}
            onRetry={async () => {
              setIsLoading(true);
              setError(null);
              try {
                const [rulesData, violationsData, complianceData] = await Promise.all([
                  getRules({ rule_type: filter.ruleType, is_active: filter.isActive }),
                  getViolations({ status: filter.violationStatus }),
                  getComplianceStats(),
                ]);
                setRules(rulesData);
                setViolations(violationsData);
                setCompliance(complianceData);
              } catch (err) {
                setError(err instanceof Error ? err : new Error("Failed to load rules and governance data"));
              } finally {
                setIsLoading(false);
              }
            }}
          />
        </div>
      </div>
    );
  }

  const openViolations = violations.filter((v) => v.status === 'open');
  const resolvedViolations = violations.filter((v) => v.status === 'resolved');

  return (
    <div className={styles.rulesGovernancePage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.headerTitle}>Rules & Governance</h1>
          <p className={styles.headerDescription}>
            Manage coding standards, quality gates, and compliance policies
          </p>
        </div>

        {/* Compliance Overview */}
        {compliance && (
          <div className={styles.metricsGrid}>
            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Compliance Score</div>
              <div className={styles.metricValue} style={{ color: (compliance.compliance_score || 0) >= 90 ? '#22c55e' : (compliance.compliance_score || 0) >= 70 ? '#eab308' : '#ef4444' }}>
                {((compliance.compliance_score || 0)).toFixed(1)}%
              </div>
            </BentoPanel>

            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Active Rules</div>
              <div className={styles.metricValue}>{compliance.active_rules}</div>
            </BentoPanel>

            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Open Violations</div>
              <div className={styles.metricValue} style={{ color: openViolations.length > 0 ? '#ef4444' : '#22c55e' }}>
                {openViolations.length}
              </div>
            </BentoPanel>

            <BentoPanel className={styles.metricCard}>
              <div className={styles.metricLabel}>Resolved Violations</div>
              <div className={styles.metricValue}>{resolvedViolations.length}</div>
            </BentoPanel>
          </div>
        )}

        {/* Filters */}
        <div className={styles.controls}>
          <div className={styles.controlGroup}>
            <label htmlFor="ruleType" className={styles.controlLabel}>
              Rule Type:
            </label>
            <select
              id="ruleType"
              value={filter.ruleType ?? ""}
              onChange={(e) => setFilter({ ...filter, ruleType: e.target.value || undefined })}
              className={styles.select}
            >
              <option value="">All Types</option>
              <option value="code_quality">Code Quality</option>
              <option value="security">Security</option>
              <option value="performance">Performance</option>
              <option value="documentation">Documentation</option>
            </select>
          </div>

          <div className={styles.controlGroup}>
            <label htmlFor="isActive" className={styles.controlLabel}>
              Status:
            </label>
            <select
              id="isActive"
              value={filter.isActive === undefined ? "" : String(filter.isActive)}
              onChange={(e) => setFilter({ ...filter, isActive: e.target.value === "" ? undefined : e.target.value === "true" })}
              className={styles.select}
            >
              <option value="">All Rules</option>
              <option value="true">Active Only</option>
              <option value="false">Inactive Only</option>
            </select>
          </div>

          <div className={styles.controlGroup}>
            <label htmlFor="violationStatus" className={styles.controlLabel}>
              Violation Status:
            </label>
            <select
              id="violationStatus"
              value={filter.violationStatus ?? ""}
              onChange={(e) => setFilter({ ...filter, violationStatus: e.target.value || undefined })}
              className={styles.select}
            >
              <option value="">All Statuses</option>
              <option value="open">Open</option>
              <option value="resolved">Resolved</option>
              <option value="ignored">Ignored</option>
            </select>
          </div>
        </div>

        {/* Rules List */}
        <BentoPanel className={styles.section}>
          <h2 className={styles.sectionTitle}>Rules ({rules.length})</h2>
          {rules.length === 0 ? (
            <div className={styles.emptyState}>
              <p className={styles.emptyText}>No rules found</p>
              <p className={styles.emptyDescription}>
                Rules will appear here once they are created.
              </p>
            </div>
          ) : (
            <div className={styles.rulesList}>
              {rules.map((rule) => (
                <div
                  key={rule.id}
                  className={`${styles.ruleItem} ${selectedRule?.id === rule.id ? styles.ruleItemSelected : ''}`}
                  onClick={() => setSelectedRule(rule)}
                >
                  <div className={styles.ruleHeader}>
                    <div className={styles.ruleTitleRow}>
                      <span className={styles.ruleName}>{rule.name}</span>
                      <div className={styles.ruleBadges}>
                        {rule.is_active ? (
                          <span className={styles.badgeActive}>Active</span>
                        ) : (
                          <span className={styles.badgeInactive}>Inactive</span>
                        )}
                        <span className={styles.badgeSeverity} data-severity={rule.severity.toLowerCase()}>
                          {rule.severity}
                        </span>
                      </div>
                    </div>
                    <span className={styles.ruleId}>{rule.id}</span>
                  </div>
                  <p className={styles.ruleDescription}>{rule.description}</p>
                  <div className={styles.ruleMeta}>
                    <span className={styles.ruleType}>{rule.rule_type}</span>
                    {rule.constitutional_reference && (
                      <span className={styles.ruleReference}>
                        Ref: {rule.constitutional_reference}
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </BentoPanel>

        {/* Violations List */}
        <BentoPanel className={styles.section}>
          <h2 className={styles.sectionTitle}>Violations ({violations.length})</h2>
          {violations.length === 0 ? (
            <div className={styles.emptyState}>
              <p className={styles.emptyText}>No violations found</p>
              <p className={styles.emptyDescription}>
                All rules are being followed correctly.
              </p>
            </div>
          ) : (
            <div className={styles.violationsList}>
              {violations.map((violation) => (
                <div key={violation.id} className={styles.violationItem}>
                  <div className={styles.violationHeader}>
                    <div className={styles.violationTitleRow}>
                      <span className={styles.violationCode}>{violation.violation_code}</span>
                      <div className={styles.violationBadges}>
                        <span className={styles.badgeStatus} data-status={violation.status.toLowerCase()}>
                          {violation.status}
                        </span>
                        <span className={styles.badgeSeverity} data-severity={violation.severity.toLowerCase()}>
                          {violation.severity}
                        </span>
                      </div>
                    </div>
                  </div>
                  <p className={styles.violationDescription}>{violation.description}</p>
                  <div className={styles.violationMeta}>
                    <span className={styles.violationRule}>Rule: {violation.rule_id}</span>
                    {violation.file_path && (
                      <span className={styles.violationLocation}>
                        {violation.file_path}
                        {violation.line_number && `:${violation.line_number}`}
                      </span>
                    )}
                    <span className={styles.violationDate}>
                      {new Date(violation.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </BentoPanel>

        {/* Violations by Severity */}
        {compliance && compliance.violations_by_severity && Object.keys(compliance.violations_by_severity).length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Violations by Severity</h2>
            <div className={styles.severityList}>
              {Object.entries(compliance.violations_by_severity).map(([severity, count]) => (
                <div key={severity} className={styles.severityItem}>
                  <span className={styles.severityName}>{severity}</span>
                  <span className={styles.severityCount}>{count}</span>
                </div>
              ))}
            </div>
          </BentoPanel>
        )}

        {/* Most Violated Rules */}
        {compliance && compliance.violations_by_rule && compliance.violations_by_rule.length > 0 && (
          <BentoPanel className={styles.section}>
            <h2 className={styles.sectionTitle}>Most Violated Rules</h2>
            <div className={styles.topViolationsList}>
              {compliance.violations_by_rule
                .sort((a, b) => b.violation_count - a.violation_count)
                .slice(0, 10)
                .map((item) => (
                  <div key={item.rule_id} className={styles.topViolationItem}>
                    <div className={styles.topViolationHeader}>
                      <span className={styles.topViolationRuleName}>{item.rule_name}</span>
                      <span className={styles.topViolationCount}>{item.violation_count}</span>
                    </div>
                    <span className={styles.topViolationRuleId}>{item.rule_id}</span>
                  </div>
                ))}
            </div>
          </BentoPanel>
        )}

        {/* Note about advanced features */}
        <BentoPanel className={styles.section}>
          <h2 className={styles.sectionTitle}>Note</h2>
          <p className={styles.infoText}>
            Rule creation, editing, and testing interfaces will be added in a future update.
            Currently, you can view existing rules and violations.
          </p>
        </BentoPanel>
      </div>
    </div>
  );
}
