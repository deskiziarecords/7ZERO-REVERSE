use crate::neuro_symbolic::{build_semantic_graph, extract_relations};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Enums & Structs ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolType {
    Perception,   // encode_candles
    Reasoning,    // query_patterns
    Decision,     // validate_signal
    Execution,    // get_trade_recommendation
    Introspection, // analyze_regime
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub reasoning: String,
    pub confidence: f64,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysis {
    pub intent_type: String,
    pub relations: Vec<String>,
    pub entities: Entities,
    pub complexity: usize,
    pub requires_introspection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entities {
    pub symbols: Vec<String>,
    pub patterns: Vec<String>,
    pub actions: Vec<String>,
    pub timeframes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub tool: String,
    pub result: serde_json::Value,
    pub quality: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOutcome {
    pub results: Vec<ExecutionResult>,
    pub final_confidence: f64,
    pub plan_completed: bool,
    pub adaptations_made: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEntry {
    pub confidence: f64,
    pub success: bool,
    pub timestamp: String,
}

// --- MetaCognitiveToolSelector ---

pub struct MetaCognitiveToolSelector {
    tool_history: Vec<String>,
    performance_memory: HashMap<String, Vec<PerformanceEntry>>,
    current_plan: Vec<ToolCall>,
}

impl MetaCognitiveToolSelector {
    pub fn new() -> Self {
        Self {
            tool_history: Vec::new(),
            performance_memory: HashMap::new(),
            current_plan: Vec::new(),
        }
    }

    /// Step 1: Semantic parsing of user intent
    pub fn analyze_intent(&self, user_query: &str) -> IntentAnalysis {
        let graph = build_semantic_graph(user_query);
        let relations = extract_relations(&graph);

        let intent_type = self.classify_intent(user_query, &relations);
        let entities = self.extract_entities(user_query);
        let complexity = relations.len();
        let requires_introspection = self.needs_introspection(user_query);

        IntentAnalysis {
            intent_type,
            relations,
            entities,
            complexity,
            requires_introspection,
        }
    }

    fn classify_intent(&self, query: &str, _relations: &[String]) -> String {
        let query_lower = query.to_lowercase();
        
        if query_lower.contains("encode") || query_lower.contains("convert") || query_lower.contains("pattern") {
            return "PERCEPTION".to_string();
        } else if query_lower.contains("query") || query_lower.contains("search") || query_lower.contains("similar") {
            return "REASONING".to_string();
        } else if query_lower.contains("validate") || query_lower.contains("check") || query_lower.contains("should i") {
            return "DECISION".to_string();
        } else if query_lower.contains("trade") || query_lower.contains("buy") || query_lower.contains("sell") {
            return "EXECUTION".to_string();
        } else if query_lower.contains("analyze") || query_lower.contains("regime") {
            return "INTROSPECTION".to_string();
        }
        "COMPOSITE".to_string()
    }

    fn extract_entities(&self, query: &str) -> Entities {
        // Regex for patterns (IXwXB...)
        let pattern_re = Regex::new(r"[IXBUDWw]{3,}").unwrap();
        let patterns: Vec<String> = pattern_re.find_iter(query).map(|m| m.as_str().to_string()).collect();

        // Regex for symbols (EURUSD)
        let symbol_re = Regex::new(r"\b[A-Z]{6,7}\b").unwrap();
        let symbols: Vec<String> = symbol_re.find_iter(query).map(|m| m.as_str().to_string()).collect();

        let actions = ["buy", "sell", "hold", "validate", "check", "encode"];
        let found_actions: Vec<String> = actions
            .iter()
            .filter(|&a| query.to_lowercase().contains(a))
            .map(|&s| s.to_uppercase())
            .collect();

        Entities {
            symbols,
            patterns,
            actions: found_actions,
            timeframes: vec![], // Simplified
        }
    }

    fn needs_introspection(&self, query: &str) -> bool {
        let triggers = ["why", "how did you", "explain", "what happened", "confidence", "doubt"];
        triggers.iter().any(|t| query.to_lowercase().contains(t))
    }

    /// Step 2: Generate tool execution plan
    pub fn generate_tool_plan(&self, intent: &IntentAnalysis) -> Vec<ToolCall> {
        let mut plan = Vec::new();
        let entities = &intent.entities;

        match intent.intent_type.as_str() {
            "PERCEPTION" if !entities.patterns.is_empty() => {
                plan.push(ToolCall {
                    tool_name: "encode_candles".to_string(),
                    parameters: json!({"ohlcv_data": "INPUT_DATA"}),
                    reasoning: format!("User wants to encode {} pattern(s)", entities.patterns.len()),
                    confidence: 0.9,
                    expected_outcome: "Symbolic pattern sequence".to_string(),
                });
            }
            "REASONING" if !entities.patterns.is_empty() => {
                plan.push(ToolCall {
                    tool_name: "query_patterns".to_string(),
                    parameters: json!({"pattern_sequence": entities.patterns.get(0).unwrap_or(&"".to_string())}),
                    reasoning: "Query vector DB for similar historical patterns".to_string(),
                    confidence: 0.85,
                    expected_outcome: "Ensemble prediction with confidence scores".to_string(),
                });
            }
            "DECISION" => {
                plan.push(ToolCall {
                    tool_name: "validate_signal".to_string(),
                    parameters: json!({
                        "pattern_sequence": entities.patterns.get(0).unwrap_or(&"".to_string()),
                        "proposed_action": entities.actions.get(0).unwrap_or(&"HOLD".to_string())
                    }),
                    reasoning: format!("Validate {} against pattern history", entities.actions.get(0).unwrap_or(&"UNKNOWN".to_string())),
                    confidence: 0.8,
                    expected_outcome: "Validation result with recommendation".to_string(),
                });
            }
            "EXECUTION" => {
                // Composite: analyze regime -> validate -> recommend
                plan.push(ToolCall {
                    tool_name: "analyze_regime".to_string(),
                    parameters: json!({"pattern_sequence": entities.patterns.get(0).unwrap_or(&"".to_string())}),
                    reasoning: "First: assess market regime for position sizing".to_string(),
                    confidence: 0.9,
                    expected_outcome: "Regime classification (TRENDING/CHOP/ALGORITHMIC)".to_string(),
                });
                plan.push(ToolCall {
                    tool_name: "get_trade_recommendation".to_string(),
                    parameters: json!({
                        "symbol": entities.symbols.get(0).unwrap_or(&"EURUSD".to_string()),
                        "current_patterns": entities.patterns.get(0).unwrap_or(&"".to_string())
                    }),
                    reasoning: "Generate complete trade plan with sizing".to_string(),
                    confidence: 0.85,
                    expected_outcome: "Trade recommendation with position size".to_string(),
                });
            }
            "INTROSPECTION" => {
                plan.push(ToolCall {
                    tool_name: "analyze_regime".to_string(),
                    parameters: json!({"pattern_sequence": entities.patterns.get(0).unwrap_or(&"".to_string())}),
                    reasoning: "Self-analysis of current market state".to_string(),
                    confidence: 0.9,
                    expected_outcome: "Regime metrics and stability assessment".to_string(),
                });
            }
            _ => { // COMPOSITE or unknown
                if entities.patterns.is_empty() {
                    plan.push(ToolCall {
                        tool_name: "encode_candles".to_string(),
                        parameters: json!({"ohlcv_data": "INPUT_DATA"}),
                        reasoning: "No patterns provided - must encode first".to_string(),
                        confidence: 0.9,
                        expected_outcome: "Pattern sequence from raw data".to_string(),
                    });
                }
                // Standard fallback pipeline
                plan.push(ToolCall {
                    tool_name: "query_patterns".to_string(),
                    parameters: json!({"pattern_sequence": "FROM_PREVIOUS"}),
                    reasoning: "Retrieve historical context".to_string(),
                    confidence: 0.85,
                    expected_outcome: "Pattern matches".to_string(),
                });
                plan.push(ToolCall {
                    tool_name: "validate_signal".to_string(),
                    parameters: json!({"pattern_sequence": "FROM_PREVIOUS", "proposed_action": "INFERRED"}),
                    reasoning: "Check signal validity".to_string(),
                    confidence: 0.8,
                    expected_outcome: "Validation result".to_string(),
                });
                plan.push(ToolCall {
                    tool_name: "get_trade_recommendation".to_string(),
                    parameters: json!({"symbol": "INFERRED", "current_patterns": "FROM_PREVIOUS"}),
                    reasoning: "Final recommendation".to_string(),
                    confidence: 0.85,
                    expected_outcome: "Complete trade plan".to_string(),
                });
            }
        }

        if intent.requires_introspection {
            plan.push(ToolCall {
                tool_name: "analyze_regime".to_string(),
                parameters: json!({"pattern_sequence": "FROM_PREVIOUS"}),
                reasoning: "Metacognitive: analyze why this decision was made".to_string(),
                confidence: 0.9,
                expected_outcome: "Explanation of market conditions".to_string(),
            });
        }

        plan
    }

    /// Step 3: Execute tool plan (Actor-Critic pattern)
    pub fn execute_plan(&mut self, plan: &[ToolCall]) -> PlanOutcome {
        let mut results = Vec::new();
        let mut cumulative_confidence = 1.0;
        let mut adaptations = 0;

        for (_i, tool_call) in plan.iter().enumerate() {
            // Actor: Execute
            let result = self.simulate_tool_execution(tool_call);

            // Critic: Evaluate
            let quality = self.evaluate_result_quality(&result, &tool_call.expected_outcome);

            // Note: In this synchronous version, we skip dynamic plan insertion
            // to avoid borrowing issues, but we log low quality.
            if quality < 0.5 {
                adaptations += 1;
            }

            results.push(ExecutionResult {
                tool: tool_call.tool_name.clone(),
                result,
                quality,
                reasoning: tool_call.reasoning.clone(),
            });

            cumulative_confidence *= tool_call.confidence * quality;
        }

        PlanOutcome {
            results,
            final_confidence: cumulative_confidence,
            plan_completed: adaptations == 0,
            adaptations_made: adaptations,
        }
    }

    fn simulate_tool_execution(&self, tool_call: &ToolCall) -> serde_json::Value {
        json!({
            "tool_name": tool_call.tool_name,
            "status": "success",
            "data": format!("Simulated result from {}", tool_call.tool_name)
        })
    }

    fn evaluate_result_quality(&self, result: &serde_json::Value, _expected: &str) -> f64 {
        if result["status"] == "success" {
            0.9
        } else {
            0.3
        }
    }

    /// Step 4: Metacognitive reflection
    pub fn reflect_and_learn(&mut self, execution: &PlanOutcome, original_query: &str) {
        let entry = PerformanceEntry {
            confidence: execution.final_confidence,
            success: execution.plan_completed,
            timestamp: Utc::now().to_rfc3339(),
        };

        self.performance_memory
            .entry(original_query.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }
}

// Helper for JSON creation
use serde_json::json;
