use crate::data::{Command, Category, Platform};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// 搜索结果
pub struct SearchResult<'a> {
    pub platform: &'a Platform,
    pub category: &'a Category,
    pub command: &'a Command,
    pub score: i64,
}

/// 搜索引擎
pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default().ignore_case(),
        }
    }

    /// 搜索命令，返回匹配结果（按分数排序）
    pub fn search<'a>(
        &self,
        query: &str,
        platforms: &'a [Platform],
    ) -> Vec<SearchResult<'a>> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for platform in platforms {
            for category in &platform.categories {
                for command in &category.commands {
                    // 对命令名称和摘要进行模糊匹配
                    let name_score = self.matcher.fuzzy_match(&command.name, query);
                    let summary_score = self.matcher.fuzzy_match(&command.summary, query);

                    // 取最高分，名称匹配加权
                    let score = match (name_score, summary_score) {
                        (Some(n), Some(s)) => {
                            // 名称匹配加权 x3
                            n * 3 + s
                        }
                        (Some(n), None) => n * 3,
                        (None, Some(s)) => s,
                        (None, None) => continue,
                    };

                    results.push(SearchResult {
                        platform,
                        category,
                        command,
                        score,
                    });
                }
            }
        }

        // 按分数降序排序
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }
}
