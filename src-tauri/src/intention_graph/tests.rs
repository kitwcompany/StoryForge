//! SING 意图图模块测试

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_intention_node_atomic() {
        let node = IntentionNode::atomic("generate", "prose", "generate story prose");
        assert_eq!(node.verb, "generate");
        assert_eq!(node.object, "prose");
        assert_eq!(node.intent_type, IntentType::Atomic);
        assert_eq!(node.canonical_text(), "generate prose");
    }

    #[test]
    fn test_intention_node_increment_frequency() {
        let mut node = IntentionNode::atomic("test", "intent", "test intent");
        assert_eq!(node.frequency, 1);
        node.increment_frequency();
        assert_eq!(node.frequency, 2);
    }

    #[test]
    fn test_asset_node_new() {
        let node = AssetNode::new(AssetType::Agent, "writer", "generate prose content", Some("writer"));
        assert_eq!(node.asset_type, AssetType::Agent);
        assert_eq!(node.name, "writer");
        assert_eq!(node.capability_id, Some("writer".to_string()));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 1e-6);
    }

    #[test]
    fn test_intent_type_from_str() {
        assert_eq!("atomic".parse::<IntentType>().unwrap(), IntentType::Atomic);
        assert_eq!("compound".parse::<IntentType>().unwrap(), IntentType::Compound);
        assert_eq!("synthetic".parse::<IntentType>().unwrap(), IntentType::Synthetic);
        assert!("unknown".parse::<IntentType>().is_err());
    }

    #[test]
    fn test_asset_type_from_str() {
        assert_eq!("skill".parse::<AssetType>().unwrap(), AssetType::Skill);
        assert_eq!("agent".parse::<AssetType>().unwrap(), AssetType::Agent);
        assert_eq!("mcp_tool".parse::<AssetType>().unwrap(), AssetType::McpTool);
        assert!("unknown".parse::<AssetType>().is_err());
    }

    #[test]
    fn test_edge_type_from_str() {
        assert_eq!(
            "has_intention".parse::<IntentionAssetEdgeType>().unwrap(),
            IntentionAssetEdgeType::HasIntention
        );
        assert_eq!(
            "triggered_by".parse::<IntentionAssetEdgeType>().unwrap(),
            IntentionAssetEdgeType::TriggeredBy
        );
        assert!("unknown".parse::<IntentionAssetEdgeType>().is_err());
    }

    #[test]
    fn test_execution_graph_status_from_str() {
        assert_eq!(
            "completed".parse::<ExecutionGraphStatus>().unwrap(),
            ExecutionGraphStatus::Completed
        );
        assert_eq!(
            "failed".parse::<ExecutionGraphStatus>().unwrap(),
            ExecutionGraphStatus::Failed
        );
        assert!("unknown".parse::<ExecutionGraphStatus>().is_err());
    }

    #[test]
    fn test_intent_context() {
        let mut ctx = IntentContext::new();
        ctx.add_input("write a chapter".to_string());
        ctx.add_intention(IntentionNode::atomic("generate", "prose", "generate prose"));
        ctx.mark_executed("writer".to_string());

        assert_eq!(ctx.input_history.len(), 1);
        assert_eq!(ctx.intention_chain.len(), 1);
        assert_eq!(ctx.executed_assets.len(), 1);
        assert_eq!(ctx.intention_chain_text(), "generate prose");
    }

    #[test]
    fn test_serialize_deserialize_embedding() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let json = serialize_embedding(&embedding);
        let deserialized = deserialize_embedding(&json).unwrap();
        assert_eq!(embedding, deserialized);
    }

    #[test]
    fn test_graph_scorer_ppr() {
        let scorer = GraphScorer::default();

        // 简单图：A -> B (weight 0.5), A -> C (weight 0.5), B -> D (weight 1.0)
        let mut edges = std::collections::HashMap::new();
        edges.insert("A".to_string(), vec![
            ("B".to_string(), 0.5),
            ("C".to_string(), 0.5),
        ]);
        edges.insert("B".to_string(), vec![("D".to_string(), 1.0)]);
        edges.insert("C".to_string(), vec![]);
        edges.insert("D".to_string(), vec![]);

        let scores = scorer.ppr_propagate(&["A".to_string()], &edges);

        // A 应该有分数（作为种子节点）
        assert!(scores.get("A").unwrap_or(&0.0) > &0.0);
        // D 应该通过 B 获得一些分数
        assert!(scores.get("D").unwrap_or(&0.0) > &0.0);
    }

    #[test]
    fn test_react_action_types() {
        let discover = ReActAction::Discover {
            source_node_id: "node_1".to_string(),
            reason: "test".to_string(),
        };
        let invoke = ReActAction::Invoke {
            node_id: "node_2".to_string(),
            parameters: serde_json::json!({}),
        };
        let respond = ReActAction::Respond {
            message: "done".to_string(),
            final_outputs: serde_json::json!({}),
        };

        assert_ne!(discover, invoke);
        assert_ne!(invoke, respond);
    }
}
