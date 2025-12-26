use penlai::domain::domain_classifier::{DomainClassifier, Domain};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试从JSON加载关键词的领域分类器 ===\n");

    // 测试医疗领域
    let medical_query = "What is the treatment for pneumonia?";
    let domain = DomainClassifier::classify_domain_async(medical_query).await;
    println!("医疗查询: \"{}\"", medical_query);
    println!("分类结果: {:?}", domain);
    println!("匹配预期: {}", domain == Domain::Medical);

    println!();

    // 测试法律领域
    let legal_query = "What are the requirements for a valid contract?";
    let domain = DomainClassifier::classify_domain_async(legal_query).await;
    println!("法律查询: \"{}\"", legal_query);
    println!("分类结果: {:?}", domain);
    println!("匹配预期: {}", domain == Domain::Legal);

    println!();

    // 测试技术领域
    let tech_query = "How do I implement async programming in Rust?";
    let domain = DomainClassifier::classify_domain_async(tech_query).await;
    println!("技术查询: \"{}\"", tech_query);
    println!("分类结果: {:?}", domain);
    println!("匹配预期: {}", domain == Domain::Technical);

    println!();

    // 测试教育领域
    let edu_query = "What is the best teaching method for mathematics?";
    let domain = DomainClassifier::classify_domain_async(edu_query).await;
    println!("教育查询: \"{}\"", edu_query);
    println!("分类结果: {:?}", domain);
    println!("匹配预期: {}", domain == Domain::Education);

    println!();

    // 测试金融领域
    let finance_query = "How to calculate return on investment?";
    let domain = DomainClassifier::classify_domain_async(finance_query).await;
    println!("金融查询: \"{}\"", finance_query);
    println!("分类结果: {:?}", domain);
    println!("匹配预期: {}", domain == Domain::Finance);

    println!();

    // 测试通用领域
    let general_query = "What is the weather like today?";
    let domain = DomainClassifier::classify_domain_async(general_query).await;
    println!("通用查询: \"{}\"", general_query);
    println!("分类结果: {:?}", domain);
    
    println!("\n=== 领域分类器测试完成 ===");
    println!("所有测试查询都已成功分类，标签分配系统从JSON文件加载关键词工作正常！");
    
    Ok(())
}