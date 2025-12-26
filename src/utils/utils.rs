use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 工具函数模块 - 提供异步上下文选择控制系统中的通用工具函数

/// 计算文本相似度的工具函数
pub mod similarity {
    /// 计算两个字符串的Jaccard相似度
    pub fn jaccard_similarity(text1: &str, text2: &str) -> f64 {
        let lower_text1 = text1.to_lowercase();
        let lower_text2 = text2.to_lowercase();
        let words1: Vec<&str> = lower_text1.split_whitespace().collect();
        let words2: Vec<&str> = lower_text2.split_whitespace().collect();

        let set1: std::collections::HashSet<&str> = words1.into_iter().collect();
        let set2: std::collections::HashSet<&str> = words2.into_iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// 计算两个字符串的余弦相似度（简化版）
    pub fn cosine_similarity(text1: &str, text2: &str) -> f64 {
        let lower_text1 = text1.to_lowercase();
        let lower_text2 = text2.to_lowercase();
        let words1: Vec<&str> = lower_text1.split_whitespace().collect();
        let words2: Vec<&str> = lower_text2.split_whitespace().collect();

        let mut word_count1 = std::collections::HashMap::new();
        let mut word_count2 = std::collections::HashMap::new();

        for word in words1 {
            *word_count1.entry(word).or_insert(0) += 1;
        }

        for word in words2 {
            *word_count2.entry(word).or_insert(0) += 1;
        }

        // 计算点积
        let mut dot_product = 0;
        for (word, count1) in &word_count1 {
            if let Some(count2) = word_count2.get(word) {
                dot_product += count1 * count2;
            }
        }

        // 计算模长
        let magnitude1 = word_count1.values().map(|count| count * count).sum::<i32>() as f64;
        let magnitude2 = word_count2.values().map(|count| count * count).sum::<i32>() as f64;

        let magnitude = (magnitude1 * magnitude2).sqrt();

        if magnitude == 0.0 {
            0.0
        } else {
            dot_product as f64 / magnitude
        }
    }
}

/// 时间相关的工具函数
pub mod time_utils {
    use chrono::{DateTime, Utc, Duration};

    /// 计算两个时间点之间的持续时间（毫秒）
    pub fn duration_between(start: DateTime<Utc>, end: DateTime<Utc>) -> f64 {
        let duration = end - start;
        duration.num_milliseconds() as f64
    }

    /// 检查时间戳是否过期
    pub fn is_expired(timestamp: DateTime<Utc>, ttl_seconds: i64) -> bool {
        let now = Utc::now();
        let ttl_duration = Duration::seconds(ttl_seconds);
        (now - timestamp) > ttl_duration
    }
}

/// 字符串处理工具函数
pub mod string_utils {
    /// 将文本按句子分割
    pub fn split_into_sentences(text: &str) -> Vec<String> {
        text.split(&['.', '!', '?', '\n'][..])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 提取文本中的关键词
    pub fn extract_keywords(text: &str, max_keywords: usize) -> Vec<String> {
        let lower_text = text.to_lowercase();
        let words: Vec<&str> = lower_text
            .split_whitespace()
            .filter(|word| word.len() > 2) // 过滤掉长度小于3的词
            .collect();

        let mut word_count = std::collections::HashMap::new();
        for word in words {
            *word_count.entry(word).or_insert(0) += 1;
        }

        let mut word_freq: Vec<(String, usize)> = word_count
            .into_iter()
            .map(|(word, count)| (word.to_string(), count))
            .collect();

        // 按频率排序
        word_freq.sort_by(|a, b| b.1.cmp(&a.1));

        word_freq
            .into_iter()
            .take(max_keywords)
            .map(|(word, _)| word)
            .collect()
    }
}

/// 数据结构相关的工具函数
pub mod data_structures {
    use std::collections::{HashMap, HashSet};

    /// 从向量中去重
    pub fn deduplicate<T: Clone + std::hash::Hash + Eq>(items: Vec<T>) -> Vec<T> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in items {
            if seen.insert(item.clone()) {
                result.push(item);
            }
        }

        result
    }

    /// 合并两个哈希映射，第二个映射的值会覆盖第一个
    pub fn merge_maps<K, V>(mut map1: HashMap<K, V>, map2: HashMap<K, V>) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq,
    {
        for (key, value) in map2 {
            map1.insert(key, value);
        }
        map1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_similarity() {
        let similarity = similarity::jaccard_similarity("hello world", "hello universe");
        assert!(similarity > 0.0);
        assert!(similarity <= 1.0);

        let similarity = similarity::jaccard_similarity("hello world", "goodbye moon");
        assert!(similarity < 0.5);
    }

    #[test]
    fn test_cosine_similarity() {
        let similarity = similarity::cosine_similarity("hello world", "hello world");
        assert!((similarity - 1.0).abs() < 0.001);

        let similarity = similarity::cosine_similarity("hello world", "goodbye moon");
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_sentence_splitting() {
        let text = "Hello world. How are you? I am fine!";
        let sentences = string_utils::split_into_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world");
        assert_eq!(sentences[1], "How are you");
        assert_eq!(sentences[2], "I am fine");
    }

    #[test]
    fn test_keyword_extraction() {
        let text = "The quick brown fox jumps over the lazy dog. The dog was really lazy.";
        let keywords = string_utils::extract_keywords(text, 3);
        // 测试关键词提取 - 由于算法可能返回不同的关键词，我们只检查长度和一些可能的关键词
        assert_eq!(keywords.len(), 3);
        // 检查是否包含一些可能的关键词
        let expected_keywords = ["the", "lazy", "dog", "really"];
        let contains_expected = keywords.iter().any(|k|
            expected_keywords.contains(&k.as_str())
        );
        assert!(contains_expected);
    }

    #[test]
    fn test_deduplication() {
        let items = vec!["a", "b", "a", "c", "b"];
        let deduplicated = data_structures::deduplicate(items);
        assert_eq!(deduplicated.len(), 3);
        assert!(deduplicated.contains(&"a"));
        assert!(deduplicated.contains(&"b"));
        assert!(deduplicated.contains(&"c"));
    }
}