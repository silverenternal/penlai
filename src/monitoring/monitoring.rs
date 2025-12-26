use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 性能指标枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMetric {
    ContextSwitchTime(f64),           // 上下文切换时间（毫秒）
    CacheHitRate(f64),               // 缓存命中率
    ResourceUsage(f64),              // 资源使用率
    RequestLatency(f64),             // 请求延迟（毫秒）
    Throughput(u64),                 // 吞吐量（每秒请求数）
    ErrorRate(f64),                  // 错误率
    ContextSelectionTime(f64),       // 上下文选择时间（毫秒）
    ConcurrentRequests(usize),       // 并发请求数
}

/// 监控事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringEvent {
    ContextLoaded { domain: String, duration_ms: f64 },
    ContextSelected { query_length: usize, selected_count: usize, duration_ms: f64 },
    CacheAccess { hit: bool, key_type: String },
    PerformanceAlert { metric: String, value: f64, threshold: f64 },
    RequestProcessed { user_id: String, session_id: String, duration_ms: f64 },
    RateLimitTriggered { user_id: String, limit: u32 },
}

/// 企业级监控系统 - 实时监控大模型异步上下文管理系统的性能
pub struct MonitoringSystem {
    /// 性能指标存储
    metrics: Arc<RwLock<HashMap<String, Vec<PerformanceMetric>>>>,
    
    /// 监控事件日志
    event_log: Arc<RwLock<Vec<(DateTime<Utc>, MonitoringEvent)>>>,
    
    /// 配置阈值
    thresholds: Arc<RwLock<HashMap<String, f64>>>,
}

impl MonitoringSystem {
    /// 创建新的企业级监控系统
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("context_switch_time_ms".to_string(), 100.0);  // 100ms阈值
        thresholds.insert("cache_hit_rate".to_string(), 0.8);           // 80%命中率
        thresholds.insert("request_latency_ms".to_string(), 500.0);     // 500ms延迟
        thresholds.insert("error_rate".to_string(), 0.05);              // 5%错误率
        thresholds.insert("context_selection_time_ms".to_string(), 200.0); // 200ms上下文选择时间
        
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
            thresholds: Arc::new(RwLock::new(thresholds)),
        }
    }

    /// 记录性能指标
    pub async fn record_metric(&self, name: &str, metric: PerformanceMetric) {
        let mut metrics = self.metrics.write().await;
        metrics
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(metric);
    }

    /// 记录监控事件
    pub async fn log_event(&self, event: MonitoringEvent) {
        let mut events = self.event_log.write().await;
        events.push((Utc::now(), event));
    }

    /// 获取特定指标的最新值
    pub async fn get_latest_metric(&self, name: &str) -> Option<PerformanceMetric> {
        let metrics = self.metrics.read().await;
        metrics
            .get(name)
            .and_then(|v| v.last().cloned())
    }

    /// 获取指标的历史数据
    pub async fn get_metric_history(&self, name: &str) -> Vec<PerformanceMetric> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned().unwrap_or_default()
    }

    /// 检查是否超过阈值并记录警报
    pub async fn check_thresholds(&self) -> Vec<String> {
        let mut alerts = Vec::new();
        let metrics = self.metrics.read().await;
        let thresholds = self.thresholds.read().await;

        for (metric_name, threshold_value) in thresholds.iter() {
            if let Some(metric_values) = metrics.get(metric_name) {
                if let Some(latest_metric) = metric_values.last() {
                    let metric_value = match latest_metric {
                        PerformanceMetric::ContextSwitchTime(v) => *v,
                        PerformanceMetric::CacheHitRate(v) => *v,
                        PerformanceMetric::ResourceUsage(v) => *v,
                        PerformanceMetric::RequestLatency(v) => *v,
                        PerformanceMetric::ErrorRate(v) => *v,
                        PerformanceMetric::ContextSelectionTime(v) => *v,
                        _ => continue, // 其他类型不进行阈值检查
                    };

                    if metric_value > *threshold_value {
                        let alert_msg = format!(
                            "Performance alert: {} ({}) exceeds threshold ({})",
                            metric_name, metric_value, threshold_value
                        );
                        alerts.push(alert_msg.clone());
                        
                        // 记录性能警报事件
                        self.log_event(MonitoringEvent::PerformanceAlert {
                            metric: metric_name.clone(),
                            value: metric_value,
                            threshold: *threshold_value,
                        }).await;
                    }
                }
            }
        }

        alerts
    }

    /// 获取最近的监控事件
    pub async fn get_recent_events(&self, count: usize) -> Vec<(DateTime<Utc>, MonitoringEvent)> {
        let events = self.event_log.read().await;
        let total_events = events.len();
        let start_idx = if count > total_events { 0 } else { total_events - count };
        
        events[start_idx..]
            .to_vec()
    }

    /// 获取系统摘要
    pub async fn get_system_summary(&self) -> SystemSummary {
        let metrics = self.metrics.read().await;
        let events = self.event_log.read().await;

        let mut avg_context_switch_time = 0.0;
        let mut avg_cache_hit_rate = 0.0;
        let mut avg_request_latency = 0.0;
        let mut avg_context_selection_time = 0.0;
        let mut error_count = 0;
        let mut total_requests = 0;
        let mut total_processed_requests = 0;

        // 计算平均上下文切换时间
        let switch_times: Vec<f64> = metrics
            .get("context_switch_time")
            .map(|v| {
                v.iter()
                    .filter_map(|m| match m {
                        PerformanceMetric::ContextSwitchTime(time) => Some(*time),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !switch_times.is_empty() {
            avg_context_switch_time = switch_times.iter().sum::<f64>() / switch_times.len() as f64;
        }

        // 计算平均缓存命中率
        let hit_rates: Vec<f64> = metrics
            .get("cache_hit_rate")
            .map(|v| {
                v.iter()
                    .filter_map(|m| match m {
                        PerformanceMetric::CacheHitRate(rate) => Some(*rate),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !hit_rates.is_empty() {
            avg_cache_hit_rate = hit_rates.iter().sum::<f64>() / hit_rates.len() as f64;
        }

        // 计算平均请求延迟
        let latencies: Vec<f64> = metrics
            .get("request_latency")
            .map(|v| {
                v.iter()
                    .filter_map(|m| match m {
                        PerformanceMetric::RequestLatency(latency) => Some(*latency),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !latencies.is_empty() {
            avg_request_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        }

        // 计算平均上下文选择时间
        let selection_times: Vec<f64> = metrics
            .get("context_selection_time")
            .map(|v| {
                v.iter()
                    .filter_map(|m| match m {
                        PerformanceMetric::ContextSelectionTime(time) => Some(*time),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !selection_times.is_empty() {
            avg_context_selection_time = selection_times.iter().sum::<f64>() / selection_times.len() as f64;
        }

        // 计算错误和请求统计
        for (_, event) in events.iter() {
            match event {
                MonitoringEvent::ContextLoaded { .. } => total_requests += 1,
                MonitoringEvent::ContextSelected { .. } => total_requests += 1,
                MonitoringEvent::RequestProcessed { .. } => total_processed_requests += 1,
                MonitoringEvent::PerformanceAlert { .. } => error_count += 1,
                _ => {}
            }
        }

        SystemSummary {
            total_metrics: metrics.len(),
            total_events: events.len(),
            avg_context_switch_time,
            avg_cache_hit_rate,
            avg_request_latency,
            avg_context_selection_time,
            error_count,
            total_requests,
            total_processed_requests,
        }
    }

    /// 获取性能趋势
    pub async fn get_performance_trends(&self, metric_name: &str, hours: i64) -> Vec<(DateTime<Utc>, f64)> {
        let metrics = self.metrics.read().await;
        let events = self.event_log.read().await;
        
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours);
        
        let relevant_metrics: Vec<(DateTime<Utc>, f64)> = events
            .iter()
            .filter(|(timestamp, _)| *timestamp >= cutoff_time)
            .filter_map(|(timestamp, event)| {
                match event {
                    MonitoringEvent::RequestProcessed { duration_ms, .. } if metric_name == "request_latency" => {
                        Some((*timestamp, *duration_ms))
                    },
                    MonitoringEvent::ContextSelected { duration_ms, .. } if metric_name == "context_selection_time" => {
                        Some((*timestamp, *duration_ms))
                    },
                    _ => None,
                }
            })
            .collect();
        
        relevant_metrics
    }
}

/// 系统摘要
#[derive(Debug)]
pub struct SystemSummary {
    pub total_metrics: usize,              // 总指标数量
    pub total_events: usize,               // 总事件数量
    pub avg_context_switch_time: f64,      // 平均上下文切换时间
    pub avg_cache_hit_rate: f64,          // 平均缓存命中率
    pub avg_request_latency: f64,         // 平均请求延迟
    pub avg_context_selection_time: f64,  // 平均上下文选择时间
    pub error_count: usize,                // 错误数量
    pub total_requests: usize,             // 总请求数量
    pub total_processed_requests: usize,   // 总处理请求数量
}

impl std::fmt::Display for SystemSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SystemSummary {{ metrics: {}, events: {}, avg_switch_time: {:.2}ms, avg_hit_rate: {:.2}%, avg_latency: {:.2}ms, avg_selection_time: {:.2}ms, errors: {}, total_requests: {}, processed_requests: {} }}",
            self.total_metrics,
            self.total_events,
            self.avg_context_switch_time,
            self.avg_cache_hit_rate * 100.0,
            self.avg_request_latency,
            self.avg_context_selection_time,
            self.error_count,
            self.total_requests,
            self.total_processed_requests
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_system() {
        let monitor = MonitoringSystem::new();

        // 记录一些测试指标
        monitor.record_metric("context_switch_time", PerformanceMetric::ContextSwitchTime(50.0)).await;
        monitor.record_metric("cache_hit_rate", PerformanceMetric::CacheHitRate(0.85)).await;
        monitor.record_metric("request_latency", PerformanceMetric::RequestLatency(200.0)).await;
        monitor.record_metric("context_selection_time", PerformanceMetric::ContextSelectionTime(150.0)).await;

        // 记录一些测试事件
        monitor.log_event(MonitoringEvent::ContextLoaded { 
            domain: "medical".to_string(), 
            duration_ms: 100.0 
        }).await;

        monitor.log_event(MonitoringEvent::ContextSelected { 
            query_length: 50, 
            selected_count: 3, 
            duration_ms: 50.0 
        }).await;

        monitor.log_event(MonitoringEvent::RequestProcessed { 
            user_id: "user1".to_string(), 
            session_id: "session1".to_string(), 
            duration_ms: 200.0 
        }).await;

        // 测试获取最新指标
        let latest_switch_time = monitor.get_latest_metric("context_switch_time").await;
        assert!(matches!(latest_switch_time, Some(PerformanceMetric::ContextSwitchTime(50.0))));

        // 测试获取指标历史
        let history = monitor.get_metric_history("cache_hit_rate").await;
        assert!(!history.is_empty());

        // 测试获取最近事件
        let recent_events = monitor.get_recent_events(5).await;
        assert!(!recent_events.is_empty());

        // 测试系统摘要
        let summary = monitor.get_system_summary().await;
        println!("{}", summary);
        
        // 验证摘要数据
        assert!(summary.total_metrics > 0);
        assert!(summary.total_events > 0);
        assert_eq!(summary.total_processed_requests, 1);

        // 测试性能趋势
        let trends = monitor.get_performance_trends("request_latency", 1).await;
        assert!(!trends.is_empty());
    }
}