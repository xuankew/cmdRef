use std::collections::{BTreeMap, BTreeSet};

use super::snapshot::{CustomCommandEntry, PasswordEntry, PromptEntry, SyncData};
use crate::bookmarks::BookmarkEntry;

/// 冲突时保留哪一侧
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prefer {
    Remote,
    Local,
}

/// 冲突明细（条目名，用于用户提示）
#[derive(Debug, Default)]
pub struct MergeReport {
    pub bookmark_conflicts: Vec<String>,
    pub command_conflicts: Vec<String>,
    pub prompt_conflicts: Vec<String>,
    pub password_conflicts: Vec<String>,
}

impl MergeReport {
    pub fn total(&self) -> usize {
        self.bookmark_conflicts.len()
            + self.command_conflicts.len()
            + self.prompt_conflicts.len()
            + self.password_conflicts.len()
    }
}

/// 条目级 3-way 合并。对每个键（三方键并集）：
/// 1. local == remote → 直接采用（双方收敛，含同时删除/同时新增相同内容）
/// 2. local == base   → 采用 remote（本地未动，远端变更生效）
/// 3. remote == base  → 采用 local（远端未动，本地变更生效）
/// 4. 其余            → 冲突，按 prefer 解决（该侧为 None 即删除）
pub fn merge_map<K, V>(
    base: &BTreeMap<K, V>,
    local: &BTreeMap<K, V>,
    remote: &BTreeMap<K, V>,
    prefer: Prefer,
) -> (BTreeMap<K, V>, Vec<K>)
where
    K: Ord + Clone,
    V: PartialEq + Clone,
{
    let mut seen: BTreeSet<K> = BTreeSet::new();
    let mut merged = BTreeMap::new();
    let mut conflicts = Vec::new();

    for map in [base, local, remote] {
        for key in map.keys() {
            if !seen.insert(key.clone()) {
                continue;
            }
            let b = base.get(key);
            let l = local.get(key);
            let r = remote.get(key);

            let chosen: Option<&V> = if l == r {
                l
            } else if l == b {
                r
            } else if r == b {
                l
            } else {
                conflicts.push(key.clone());
                match prefer {
                    Prefer::Remote => r,
                    Prefer::Local => l,
                }
            };

            if let Some(v) = chosen {
                merged.insert(key.clone(), v.clone());
            }
        }
    }

    (merged, conflicts)
}

/// 对三类可同步数据分别做条目级 3-way 合并
pub fn merge_sync_data(
    base: &SyncData,
    local: &SyncData,
    remote: &SyncData,
    prefer: Prefer,
) -> (SyncData, MergeReport) {
    let mut report = MergeReport::default();

    let bm_base = index_bookmarks(&base.bookmarks);
    let bm_local = index_bookmarks(&local.bookmarks);
    let bm_remote = index_bookmarks(&remote.bookmarks);
    let (bm_merged, bm_conflicts) = merge_map(&bm_base, &bm_local, &bm_remote, prefer);
    report.bookmark_conflicts = bm_conflicts.into_iter().map(|(_, _, cmd)| cmd).collect();

    let cmd_base = index_commands(&base.custom_commands);
    let cmd_local = index_commands(&local.custom_commands);
    let cmd_remote = index_commands(&remote.custom_commands);
    let (cmd_merged, cmd_conflicts) = merge_map(&cmd_base, &cmd_local, &cmd_remote, prefer);
    report.command_conflicts = cmd_conflicts.into_iter().map(|(_, _, name)| name).collect();

    let p_base = index_prompts(&base.prompts);
    let p_local = index_prompts(&local.prompts);
    let p_remote = index_prompts(&remote.prompts);
    let (p_merged, p_conflicts) = merge_map(&p_base, &p_local, &p_remote, prefer);
    report.prompt_conflicts = p_conflicts;

    let pw_base = index_passwords(&base.passwords);
    let pw_local = index_passwords(&local.passwords);
    let pw_remote = index_passwords(&remote.passwords);
    let (pw_merged, pw_conflicts) = merge_map(&pw_base, &pw_local, &pw_remote, prefer);
    report.password_conflicts = pw_conflicts;

    // crypto_salt: 3-way scalar merge (same logic as merge_map but for a single string)
    let salt_base = &base.crypto_salt;
    let salt_local = &local.crypto_salt;
    let salt_remote = &remote.crypto_salt;
    let crypto_salt = if salt_local == salt_remote {
        salt_local.clone()
    } else if salt_local == salt_base {
        salt_remote.clone()
    } else if salt_remote == salt_base {
        salt_local.clone()
    } else {
        match prefer {
            Prefer::Remote => salt_remote.clone(),
            Prefer::Local => salt_local.clone(),
        }
    };

    (
        SyncData {
            bookmarks: bm_merged.into_values().collect(),
            custom_commands: cmd_merged.into_values().collect(),
            prompts: p_merged.into_values().collect(),
            passwords: pw_merged.into_values().collect(),
            crypto_salt,
        },
        report,
    )
}

type BookmarkKey = (String, String, String);
type CommandKey = (String, String, String);

fn index_bookmarks(list: &[BookmarkEntry]) -> BTreeMap<BookmarkKey, BookmarkEntry> {
    list.iter()
        .map(|b| {
            (
                (b.platform.clone(), b.category.clone(), b.command.clone()),
                b.clone(),
            )
        })
        .collect()
}

fn index_commands(
    list: &[CustomCommandEntry],
) -> BTreeMap<CommandKey, CustomCommandEntry> {
    list.iter()
        .map(|c| {
            (
                (c.platform.clone(), c.category.clone(), c.command.name.clone()),
                c.clone(),
            )
        })
        .collect()
}

fn index_prompts(list: &[PromptEntry]) -> BTreeMap<String, PromptEntry> {
    list.iter().map(|p| (p.name.clone(), p.clone())).collect()
}

fn index_passwords(list: &[PasswordEntry]) -> BTreeMap<String, PasswordEntry> {
    list.iter().map(|p| (p.name.clone(), p.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 3-way 合并 14 行真值表。
    /// 每行: (base, local, remote, expected, is_conflict)
    /// 值语义: 0=不存在, 其他数字=条目值（不同数字=不同内容）
    #[test]
    fn test_truth_table_14_rows() {
        // (base, local, remote, expected, conflict)
        let rows: Vec<(u8, u8, u8, u8, bool)> = vec![
            // 1. 三方一致
            (1, 1, 1, 1, false),
            // 2. 远端删除，本地未动 → 删除
            (1, 1, 0, 0, false),
            // 3. 本地删除，远端未动 → 删除
            (1, 0, 1, 0, false),
            // 4. 双方新增相同
            (0, 1, 1, 1, false),
            // 5. 远端新增，本地无
            (0, 0, 1, 1, false),
            // 6. 本地新增，远端无
            (0, 1, 0, 1, false),
            // 7. 双方删除
            (1, 0, 0, 0, false),
            // 8. 远端修改，本地未动 → 取远端
            (1, 1, 2, 2, false),
            // 9. 本地修改，远端未动 → 取本地
            (1, 2, 1, 2, false),
            // 10. 双方改成相同值
            (1, 2, 2, 2, false),
            // 11. 三方各不相同 → 冲突
            (1, 2, 3, 3, true),
            // 12. 远端修改，本地删除 → 冲突
            (1, 0, 2, 2, true),
            // 13. 本地修改，远端删除 → 冲突
            (1, 2, 0, 0, true),
            // 14. 双方新增不同值 → 冲突
            (0, 1, 2, 2, true),
        ];

        for (i, (b, l, r, expected, conflict)) in rows.iter().enumerate() {
            let base = map_of(*b);
            let local = map_of(*l);
            let remote = map_of(*r);
            let expected_opt = if *expected == 0 { None } else { Some(*expected) };

            let (merged, conflicts) =
                merge_map(&base, &local, &remote, Prefer::Remote);
            assert_eq!(
                merged.get("k").map(|v| v.value),
                expected_opt,
                "row {} (prefer remote): wrong result",
                i + 1
            );
            assert_eq!(
                conflicts.is_empty(),
                !conflict,
                "row {} (prefer remote): conflict flag mismatch",
                i + 1
            );

            // prefer local 时冲突行取本地值
            let (merged_l, conflicts_l) =
                merge_map(&base, &local, &remote, Prefer::Local);
            if *conflict {
                let local_val = if *l == 0 { None } else { Some(*l) };
                assert_eq!(
                    merged_l.get("k").map(|v| v.value),
                    local_val,
                    "row {} (prefer local): wrong result",
                    i + 1
                );
            } else {
                assert_eq!(merged_l.get("k").map(|v| v.value), expected_opt);
            }
            assert_eq!(conflicts_l.is_empty(), !conflict);
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Item {
        value: u8,
    }

    fn map_of(v: u8) -> BTreeMap<String, Item> {
        if v == 0 {
            BTreeMap::new()
        } else {
            BTreeMap::from([("k".to_string(), Item { value: v })])
        }
    }

    fn bookmark(platform: &str, category: &str, command: &str) -> BookmarkEntry {
        BookmarkEntry {
            platform: platform.into(),
            category: category.into(),
            command: command.into(),
        }
    }

    #[test]
    fn test_multiple_keys_independent() {
        // key-a 仅本地改，key-b 仅远端改，key-c 双方改不同
        let mut base = BTreeMap::new();
        base.insert("a", Item { value: 1 });
        base.insert("b", Item { value: 1 });
        base.insert("c", Item { value: 1 });

        let mut local = base.clone();
        local.insert("a", Item { value: 2 }); // 本地改 a
        local.insert("d", Item { value: 9 }); // 本地新增 d

        let mut remote = base.clone();
        remote.insert("b", Item { value: 3 }); // 远端改 b
        remote.insert("e", Item { value: 8 }); // 远端新增 e
        remote.remove("c"); // 远端删 c，本地未动 → 删除

        let (merged, conflicts) = merge_map(&base, &local, &remote, Prefer::Remote);
        assert_eq!(merged.get("a").unwrap().value, 2); // 本地改生效
        assert_eq!(merged.get("b").unwrap().value, 3); // 远端改生效
        assert!(!merged.contains_key("c")); // 远端删除生效
        assert_eq!(merged.get("d").unwrap().value, 9); // 本地新增保留
        assert_eq!(merged.get("e").unwrap().value, 8); // 远端新增保留
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_merge_sync_data_sections() {
        let base = SyncData {
            bookmarks: vec![bookmark("mac", "Homebrew", "brew install")],
            custom_commands: vec![],
            prompts: vec![PromptEntry {
                file: "my_prompts".into(),
                name: "p1".into(),
                description: String::new(),
                content: "base".into(),
                tags: vec![],
            }],
            passwords: vec![],
            crypto_salt: String::new(),
        };

        let mut local = base.clone();
        local.bookmarks.push(bookmark("dev", "Git", "git rebase")); // 本地新增书签

        let mut remote = base.clone();
        remote.prompts[0].content = "remote-edit".into(); // 远端改 prompt

        let (merged, report) =
            merge_sync_data(&base, &local, &remote, Prefer::Remote);

        assert_eq!(merged.bookmarks.len(), 2);
        assert_eq!(merged.prompts[0].content, "remote-edit");
        assert_eq!(report.total(), 0);
    }

    #[test]
    fn test_merge_sync_data_conflict_report() {
        let mk_prompt = |content: &str| PromptEntry {
            file: "my_prompts".into(),
            name: "p1".into(),
            description: String::new(),
            content: content.into(),
            tags: vec![],
        };
        let base = SyncData {
            bookmarks: vec![],
            custom_commands: vec![],
            prompts: vec![mk_prompt("base")],
            passwords: vec![],
            crypto_salt: String::new(),
        };
        let local = SyncData {
            bookmarks: vec![],
            custom_commands: vec![],
            prompts: vec![mk_prompt("local-edit")],
            passwords: vec![],
            crypto_salt: String::new(),
        };
        let remote = SyncData {
            bookmarks: vec![],
            custom_commands: vec![],
            prompts: vec![mk_prompt("remote-edit")],
            passwords: vec![],
            crypto_salt: String::new(),
        };

        let (merged, report) = merge_sync_data(&base, &local, &remote, Prefer::Remote);
        assert_eq!(merged.prompts[0].content, "remote-edit");
        assert_eq!(report.prompt_conflicts, vec!["p1".to_string()]);
        assert_eq!(report.total(), 1);

        let (merged, report) = merge_sync_data(&base, &local, &remote, Prefer::Local);
        assert_eq!(merged.prompts[0].content, "local-edit");
        assert_eq!(report.total(), 1);
    }

    /// 无基线的首次 pull：本地与远端互不冲突的条目全部保留
    #[test]
    fn test_first_pull_no_base_keeps_both_sides() {
        let empty = SyncData::default();
        let local = SyncData {
            bookmarks: vec![bookmark("dev", "Git", "git rebase")],
            custom_commands: vec![],
            prompts: vec![],
            passwords: vec![],
            crypto_salt: String::new(),
        };
        let remote = SyncData {
            bookmarks: vec![bookmark("mac", "Homebrew", "brew install")],
            custom_commands: vec![],
            prompts: vec![],
            passwords: vec![],
            crypto_salt: String::new(),
        };

        let (merged, report) = merge_sync_data(&empty, &local, &remote, Prefer::Remote);
        assert_eq!(merged.bookmarks.len(), 2);
        assert_eq!(report.total(), 0);
    }
}
