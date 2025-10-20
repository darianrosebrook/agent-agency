//! Adaptive prompting strategy that learns from evaluation feedback

use std::collections::HashMap;
use async_trait::async_trait;

use super::PromptingStrategy;
use crate::evaluation::EvalReport;
use crate::types::{Task, TaskType};

/// Template for prompts
#[derive(Debug, Clone)]
struct PromptTemplate {
    template: String,
    variables: Vec<String>,
}

/// Adaptive prompting strategy
pub struct AdaptivePromptingStrategy {
    initial_templates: HashMap<TaskType, Vec<PromptTemplate>>,
    refinement_templates: Vec<PromptTemplate>,
    learning_signals: Vec<LearningSignal>,
    success_patterns: HashMap<String, f64>,
}

impl AdaptivePromptingStrategy {
    /// Create a new adaptive prompting strategy
    pub fn new() -> Self {
        let mut initial_templates = HashMap::new();

        // Code generation templates
        initial_templates.insert(TaskType::CodeGeneration, vec![
            PromptTemplate {
                template: r#"Generate code that implements the following requirement:

Task: {description}

Requirements:
- Write clean, maintainable code
- Include proper error handling
- Add comments for complex logic
- Follow language best practices

Generate the complete implementation:"#.to_string(),
                variables: vec!["description".to_string()],
            },
        ]);

        // Code fix templates
        initial_templates.insert(TaskType::CodeFix, vec![
            PromptTemplate {
                template: r#"Fix the following code issue:

Problem: {description}

Current code to fix:
```
{code_context}
```

Requirements:
- Maintain existing functionality
- Improve code quality
- Fix any bugs or issues
- Add proper error handling if needed

Provide the corrected code:"#.to_string(),
                variables: vec!["description".to_string(), "code_context".to_string()],
            },
        ]);

        // Text transformation templates
        initial_templates.insert(TaskType::TextTransformation, vec![
            PromptTemplate {
                template: r#"Transform the following text according to these requirements:

Task: {description}

Original text:
{text_content}

Requirements:
- Maintain the core meaning
- Improve clarity and readability
- Use professional language
- Keep appropriate length

Transformed text:"#.to_string(),
                variables: vec!["description".to_string(), "text_content".to_string()],
            },
        ]);

        // Documentation templates
        initial_templates.insert(TaskType::DocumentationUpdate, vec![
            PromptTemplate {
                template: r#"Update documentation with the following changes:

Task: {description}

Current documentation:
{doc_content}

Requirements:
- Ensure accuracy and completeness
- Use clear, concise language
- Follow documentation conventions
- Include code examples where appropriate

Updated documentation:"#.to_string(),
                variables: vec!["description".to_string(), "doc_content".to_string()],
            },
        ]);

        // Design token application templates
        initial_templates.insert(TaskType::DesignTokenApplication, vec![
            PromptTemplate {
                template: r#"Apply design system tokens to the following code:

Task: {description}

Code to update:
{code_content}

Available tokens:
- Colors: primary (#0066cc), secondary (#666666), success (#28a745)
- Spacing: small (4px), medium (8px), large (16px)
- Typography: body (14px), heading (18px), caption (12px)

Requirements:
- Replace hardcoded values with token references
- Maintain visual consistency
- Ensure accessibility compliance

Updated code:"#.to_string(),
                variables: vec!["description".to_string(), "code_content".to_string()],
            },
        ]);

        // Refinement templates based on evaluation feedback
        let refinement_templates = vec![
            PromptTemplate {
                template: r#"Previous attempt scored {score:.1}%. The following issues need to be addressed:

Failed criteria:
{failed_criteria}

Specific feedback:
{feedback}

Please revise your solution to address these issues:"#.to_string(),
                variables: vec!["score".to_string(), "failed_criteria".to_string(), "feedback".to_string()],
            },
            PromptTemplate {
                template: r#"The previous implementation had these problems:
{issues}

Please provide an improved version that fixes these issues while maintaining the original requirements."#.to_string(),
                variables: vec!["issues".to_string()],
            },
        ];

        Self {
            initial_templates,
            refinement_templates,
            learning_signals: Vec::new(),
            success_patterns: HashMap::new(),
        }
    }

    /// Select best initial template for task type
    fn select_initial_template(&self, task_type: &TaskType) -> &PromptTemplate {
        self.initial_templates
            .get(task_type)
            .and_then(|templates| templates.first())
            .unwrap_or_else(|| {
                // Fallback generic template
                &PromptTemplate {
                    template: "Complete the following task: {description}".to_string(),
                    variables: vec!["description".to_string()],
                }
            })
    }

    /// Select best refinement template based on evaluation results
    fn select_refinement_template(&self, eval_report: &EvalReport) -> &PromptTemplate {
        // For now, just use the first template
        // In the future, this could be adaptive based on learning
        &self.refinement_templates[0]
    }

    /// Fill template variables
    fn fill_template(&self, template: &PromptTemplate, variables: HashMap<&str, String>) -> String {
        let mut result = template.template.clone();

        for var in &template.variables {
            if let Some(value) = variables.get(var.as_str()) {
                result = result.replace(&format!("{{{}}}", var), value);
            }
        }

        result
    }

    /// Extract context from task for template variables
    fn extract_task_context(&self, task: &Task) -> HashMap<&str, String> {
        let mut context = HashMap::new();

        context.insert("description", task.description.clone());

        // Add target file context if available
        if let Some(first_file) = task.target_files.first() {
            context.insert("target_file", first_file.clone());
        }

        // Add constraints
        let constraints_str = task.constraints.iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        if !constraints_str.is_empty() {
            context.insert("constraints", constraints_str);
        }

        context
    }

    /// Extract evaluation context for refinement
    fn extract_evaluation_context(&self, eval_report: &EvalReport) -> HashMap<&str, String> {
        let mut context = HashMap::new();

        context.insert("score", format!("{:.1}", eval_report.score * 100.0));

        // Failed criteria
        let failed_criteria = eval_report.criteria.iter()
            .filter(|c| !c.passed)
            .map(|c| format!("- {}: {}", c.id, c.notes.as_ref().unwrap_or(&"".to_string())))
            .collect::<Vec<_>>()
            .join("\n");

        context.insert("failed_criteria", failed_criteria);

        // General feedback
        let feedback = eval_report.next_actions.join("; ");
        context.insert("feedback", feedback);

        // Issues from logs
        let issues = eval_report.logs.join("; ");
        context.insert("issues", issues);

        context
    }

    /// Learn from successful patterns
    pub fn learn_from_success(&mut self, task_type: &TaskType, template_used: &str, score: f64) {
        let key = format!("{:?}:{}", task_type, template_used);
        let current = self.success_patterns.get(&key).copied().unwrap_or(0.0);
        // Exponential moving average
        let updated = 0.9 * current + 0.1 * score;
        self.success_patterns.insert(key, updated);

        self.learning_signals.push(LearningSignal::TemplateSuccess {
            template: template_used.to_string(),
            task_type: task_type.clone(),
            score,
        });
    }

    /// Learn from failures
    pub fn learn_from_failure(&mut self, task_type: &TaskType, template_used: &str, issues: Vec<String>) {
        self.learning_signals.push(LearningSignal::TemplateFailure {
            template: template_used.to_string(),
            task_type: task_type.clone(),
            issues,
        });
    }
}

#[async_trait]
impl PromptingStrategy for AdaptivePromptingStrategy {
    fn generate_initial_prompt(&self, task: &Task) -> String {
        let template = self.select_initial_template(&task.task_type);
        let context = self.extract_task_context(task);
        self.fill_template(template, context)
    }

    fn generate_refinement_prompt(&self, eval_report: &EvalReport) -> String {
        let template = self.select_refinement_template(eval_report);
        let context = self.extract_evaluation_context(eval_report);
        self.fill_template(template, context)
    }

    fn generate_self_critique_prompt(&self, output: &str) -> String {
        format!(
            r#"Please critically evaluate the following output for quality, correctness, and completeness:

Output:
{}

Evaluation criteria:
- Correctness: Does it solve the intended problem?
- Completeness: Are all requirements addressed?
- Quality: Is the code/text well-structured and maintainable?
- Best practices: Does it follow language/framework conventions?

Provide a detailed critique and suggestions for improvement:"#,
            output
        )
    }
}

/// Learning signals for adaptive prompting
#[derive(Debug, Clone)]
pub enum LearningSignal {
    TemplateSuccess {
        template: String,
        task_type: TaskType,
        score: f64,
    },
    TemplateFailure {
        template: String,
        task_type: TaskType,
        issues: Vec<String>,
    },
}
