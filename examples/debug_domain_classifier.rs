use penlai::domain::domain_classifier::{DomainClassifier, Domain};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 调试领域分类器 ===\n");

    // 测试医疗领域
    let medical_query = "What is the treatment for pneumonia?";
    println!("医疗查询: \"{}\"", medical_query);
    
    // 手动检查关键词匹配
    let keywords_json = std::fs::read_to_string("src/domain/keywords.json")?;
    let keywords: serde_json::Value = serde_json::from_str(&keywords_json)?;
    
    let medical_keywords: Vec<String> = serde_json::from_value(keywords["medical"].clone()).unwrap();
    println!("医疗关键词: {:?}", &medical_keywords[0..5]); // 显示前5个关键词
    
    // 检查查询中是否包含关键词
    let lower_query = medical_query.to_lowercase();
    let mut matches_found = Vec::new();
    for keyword in &medical_keywords {
        if lower_query.contains(keyword) {
            matches_found.push(keyword.clone());
        }
    }
    println!("找到的匹配: {:?}", matches_found);
    
    let domain = DomainClassifier::classify_domain_async(medical_query).await;
    println!("分类结果: {:?}", domain);
    println!();

    // 测试法律领域
    let legal_query = "What are the requirements for a valid contract?";
    println!("法律查询: \"{}\"", legal_query);

    let legal_keywords: Vec<String> = serde_json::from_value(keywords["legal"].clone()).unwrap();
    println!("法律关键词: {:?}", &legal_keywords[0..5]); // 显示前5个关键词

    let lower_query = legal_query.to_lowercase();
    let mut matches_found = Vec::new();
    for keyword in &legal_keywords {
        if lower_query.contains(keyword) {
            matches_found.push(keyword.clone());
        }
    }
    println!("找到的匹配: {:?}", matches_found);

    let domain = DomainClassifier::classify_domain_async(legal_query).await;
    println!("分类结果: {:?}", domain);
    println!();

    // 测试教育领域
    let edu_query = "What is the best teaching method for mathematics?";
    println!("教育查询: \"{}\"", edu_query);

    let edu_keywords: Vec<String> = serde_json::from_value(keywords["education"].clone()).unwrap();
    println!("教育关键词: {:?}", &edu_keywords[0..5]); // 显示前5个关键词

    let lower_query = edu_query.to_lowercase();
    let mut matches_found = Vec::new();
    for keyword in &edu_keywords {
        if lower_query.contains(keyword) {
            matches_found.push(keyword.clone());
        }
    }
    println!("找到的匹配: {:?}", matches_found);

    let domain = DomainClassifier::classify_domain_async(edu_query).await;
    println!("分类结果: {:?}", domain);
    println!();

    Ok(())
}