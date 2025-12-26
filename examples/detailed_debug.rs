use penlai::domain::domain_classifier::{DomainClassifier, Domain};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 详细调试领域分类器 ===\n");

    // 手动测试分类逻辑
    let medical_query = "What is the treatment for pneumonia?";
    println!("医疗查询: \"{}\"", medical_query);
    
    // 直接使用领域分类器实例来测试
    match DomainClassifier::new_instance() {
        Ok(classifier) => {
            println!("成功加载分类器，开始详细分析...");

            let text = medical_query;
            let lower_text = text.to_lowercase();
            let words: Vec<&str> = lower_text.split_whitespace().collect();

            println!("分词结果: {:?}", words);

            // 手动计算每个领域的分数
            let mut scores = std::collections::HashMap::new();
            scores.insert(Domain::Medical, 0);
            scores.insert(Domain::Legal, 0);
            scores.insert(Domain::Technical, 0);
            scores.insert(Domain::Education, 0);
            scores.insert(Domain::Finance, 0);
            scores.insert(Domain::General, 0);

            println!("\n关键词匹配分析:");

            // 检查医疗关键词
            let mut medical_matches = Vec::new();
            for keyword in &classifier.medical_keywords {
                for word in &words {
                    if word.contains(keyword) || text.to_lowercase().contains(keyword) {
                        medical_matches.push(keyword.clone());
                        *scores.get_mut(&Domain::Medical).unwrap() += 1;
                    }
                }
            }
            println!("医疗领域匹配: {:?}, 得分: {}", medical_matches, scores[&Domain::Medical]);

            // 检查法律关键词
            let mut legal_matches = Vec::new();
            for keyword in &classifier.legal_keywords {
                for word in &words {
                    if word.contains(keyword) || text.to_lowercase().contains(keyword) {
                        legal_matches.push(keyword.clone());
                        *scores.get_mut(&Domain::Legal).unwrap() += 1;
                    }
                }
            }
            println!("法律领域匹配: {:?}, 得分: {}", legal_matches, scores[&Domain::Legal]);

            // 检查技术关键词
            let mut tech_matches = Vec::new();
            for keyword in &classifier.technical_keywords {
                for word in &words {
                    if word.contains(keyword) || text.to_lowercase().contains(keyword) {
                        tech_matches.push(keyword.clone());
                        *scores.get_mut(&Domain::Technical).unwrap() += 1;
                    }
                }
            }
            println!("技术领域匹配: {:?}, 得分: {}", tech_matches, scores[&Domain::Technical]);

            // 检查教育关键词
            let mut edu_matches = Vec::new();
            for keyword in &classifier.education_keywords {
                for word in &words {
                    if word.contains(keyword) || text.to_lowercase().contains(keyword) {
                        edu_matches.push(keyword.clone());
                        *scores.get_mut(&Domain::Education).unwrap() += 1;
                    }
                }
            }
            println!("教育领域匹配: {:?}, 得分: {}", edu_matches, scores[&Domain::Education]);

            // 检查金融关键词
            let mut finance_matches = Vec::new();
            for keyword in &classifier.finance_keywords {
                for word in &words {
                    if word.contains(keyword) || text.to_lowercase().contains(keyword) {
                        finance_matches.push(keyword.clone());
                        *scores.get_mut(&Domain::Finance).unwrap() += 1;
                    }
                }
            }
            println!("金融领域匹配: {:?}, 得分: {}", finance_matches, scores[&Domain::Finance]);

            // 检查通用关键词
            let mut general_matches = Vec::new();
            for keyword in &classifier.general_keywords {
                for word in &words {
                    if word.contains(keyword) || text.to_lowercase().contains(keyword) {
                        general_matches.push(keyword.clone());
                        *scores.get_mut(&Domain::General).unwrap() += 1;
                    }
                }
            }
            println!("通用领域匹配: {:?}, 得分: {}", general_matches, scores[&Domain::General]);

            // 显示总分
            println!("\n各领域总分:");
            for (domain, score) in &scores {
                println!("  {:?}: {}", domain, score);
            }

            // 找到最高分领域
            let highest_score_domain = scores
                .into_iter()
                .max_by_key(|&(_, score)| score)
                .map(|(domain, _)| domain)
                .unwrap_or(Domain::General);

            println!("\n最终分类结果: {:?}", highest_score_domain);

        },
        Err(e) => {
            println!("无法加载分类器: {}", e);
            // 使用默认分类
            let domain = DomainClassifier::default_classify_domain(medical_query);
            println!("使用默认分类结果: {:?}", domain);
        }
    }

    // 测试异步分类方法
    println!("\n测试异步分类方法:");
    let domain = DomainClassifier::classify_domain_async(medical_query).await;
    println!("异步分类方法结果: {:?}", domain);

    Ok(())
}