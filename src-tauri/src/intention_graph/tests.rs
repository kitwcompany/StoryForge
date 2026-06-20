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

    // ================================================================
    // v0.20.1 端到端集成测试：验证 SING 意图图全流程
    //
    // 模拟用户输入"写一部异星球末世生存题材的小说"，验证：
    // 1. AssetSyncEngine 能填充资产到数据库
    // 2. LayeredDiscovery::discover 能用 PPR 发现相关资产
    // 3. 发现的资产不为空（修复前此路径 100% 返回空）
    // ================================================================

    #[test]
    fn test_e2e_asset_sync_and_discover() {
        use crate::db::connection::create_test_pool;

        // 1. 创建内存数据库（含 Migration 95 意图图表）
        let pool = create_test_pool().expect("Failed to create test pool");
        let repo = graph::IntentionGraphRepository::new(pool);

        // 2. 用 AssetSyncEngine 填充资产（模拟 lib.rs setup 阶段）
        let sync_engine = asset_sync::AssetSyncEngine::new(repo.clone());
        let registry = crate::capabilities::CapabilityRegistry::new();
        let selectable_assets: Vec<crate::strategy::SelectableAsset> = vec![];

        let stats = sync_engine
            .full_initialize(&registry, &selectable_assets)
            .expect("AssetSync failed");

        // 验证资产被填充
        assert!(stats.agents > 0, "Should sync builtin agents");
        assert!(stats.system_commands > 0, "Should sync system commands");
        assert!(stats.asset_edges > 0, "Should build asset edges");

        // 3. 模拟用户意图"写一部异星球末世生存题材的小说" → "generate prose"
        let root_intention = IntentionNode::atomic(
            "generate",
            "prose",
            "generate novel prose about alien planet survival",
        );

        // 4. PPR 分层发现
        let scorer = GraphScorer::new(scorer::PprConfig::default());
        let discovery = discovery::LayeredDiscovery::new(scorer);
        let results = discovery
            .discover(&root_intention, &repo, 10)
            .expect("Discover failed");

        // 5. 验证发现了资产（修复前此路径返回空 Vec）
        assert!(
            !results.is_empty(),
            "Should discover assets for 'generate prose' intention"
        );

        // 6. 验证 writer 资产被高排名发现
        let has_writer = results.iter().any(|r| {
            r.asset.name.contains("writer") || r.asset.id.contains("writer")
        });
        assert!(
            has_writer,
            "Writer agent should be discovered for 'generate prose' intention"
        );

        // 7. 验证评分合理（desc + intent + ppr > 0）
        let top = &results[0];
        assert!(top.score > 0.0, "Top asset score should be positive");
        assert!(
            top.intent_score > 0.0,
            "Intent match score should be positive for matching verb/object"
        );

        eprintln!(
            "[E2E] 用户输入: '写一部异星球末世生存题材的小说'");
        eprintln!(
            "[E2E] 意图合成: generate prose (confidence via rule-based)");
        eprintln!(
            "[E2E] PPR 分层发现 {} 个资产:", results.len());
        for (i, r) in results.iter().enumerate().take(5) {
            eprintln!(
                "  #{} {:?} {} (score={:.3}, {})",
                i + 1, r.asset.asset_type, r.asset.name, r.score, r.reason
            );
        }
        eprintln!(
            "[E2E] ✓ 全流程通过: AssetSync填充 → PPR发现 → writer资产排名靠前");
    }

    #[test]
    fn test_e2e_execution_graph_persistence() {
        use crate::db::connection::create_test_pool;

        let pool = create_test_pool().expect("Failed to create test pool");
        let repo = graph::IntentionGraphRepository::new(pool);

        // 创建执行图并持久化
        let graph = ExecutionGraph {
            id: "eg_test_001".to_string(),
            request_id: "req_test_001".to_string(),
            story_id: Some("story_1".to_string()),
            user_input: "写一部异星球末世生存题材的小说".to_string(),
            root_intention_id: Some("generate_prose".to_string()),
            status: ExecutionGraphStatus::Completed,
            plan_json: Some(r#"{"steps":[]}"#.to_string()),
            result_json: Some(r#"{"content":"..."}"#.to_string()),
            created_at: chrono::Local::now(),
            completed_at: Some(chrono::Local::now()),
            execution_time_ms: Some(1500),
        };

        repo.create_execution_graph(&graph).expect("Create graph failed");

        // 验证能查询回来
        let retrieved = repo
            .get_execution_graph("eg_test_001")
            .expect("Get graph failed");
        assert!(retrieved.is_some(), "Execution graph should be persisted");
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.user_input, "写一部异星球末世生存题材的小说");
        assert_eq!(retrieved.status, ExecutionGraphStatus::Completed);

        // 验证能查询最近执行（供诊断面板）
        let recent = repo.get_recent_executions(10).expect("Get recent failed");
        assert!(!recent.is_empty(), "Should have recent executions");

        // 验证统计信息
        let stats = repo.get_statistics().expect("Get stats failed");
        assert!(stats.execution_graph_count > 0, "Should count execution graphs");
    }
}
