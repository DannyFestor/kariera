//! The Skill Sheet of the fixture vault as XLSX (issue #9), read back from the file the way a spreadsheet program
//! reads it. The model comes from the cli's tests because only the cli may load a vault (ADR 0001).

use kariera_render_xlsx::render;

// Both are test code in the eyes of Clippy, which allows `expect` and indexing there.
#[cfg(test)]
#[path = "support/skill_sheet.rs"]
mod skill_sheet;
#[cfg(test)]
#[path = "support/xlsx.rs"]
mod xlsx;

use skill_sheet::{fixture_skill_sheet, project_top, rendered, skill_rows, texts_between};

#[test]
fn the_header_band_names_the_sheet_the_owner_and_the_update_month() {
    let xlsx = rendered(&fixture_skill_sheet());

    assert_eq!(
        xlsx.cell(0, 0).as_deref(),
        Some("スキルシート\u{3000}\u{3000}Robin Beispiel")
    );
    assert_eq!(xlsx.cell(0, 9).as_deref(), Some("更新日 2026年11月"));
}

#[test]
fn the_profile_table_states_each_fact_beside_its_label() {
    let xlsx = rendered(&fixture_skill_sheet());

    let facts = [
        ("得意分野", "日本在住のフルスタック開発者"),
        ("所属", "Example Works"),
        ("稼働", "2026年4月1日から\u{3000}週3～5日"),
        ("URL", "https://example.com\nhttps://github.com/example"),
        (
            "資格",
            "2022年12月 日本語試験B2\n2024年2月 Example Cloud Associate",
        ),
        ("学歴", "2018年3月 情報科学 学士"),
        (
            "得意技術",
            "・PostgreSQLを用いたバックエンド設計\n・日本語・ドイツ語・英語での業務",
        ),
    ];
    for (label, value) in facts {
        assert_eq!(xlsx.beside(label, 2).as_deref(), Some(value), "{label}");
    }
}

#[test]
fn a_fact_the_sheet_does_not_know_prints_the_unknown_mark() {
    let mut sheet = fixture_skill_sheet();
    sheet.availability = None;

    let xlsx = rendered(&sheet);

    assert_eq!(xlsx.beside("稼働", 2).as_deref(), Some("—"));
}

#[test]
fn the_skill_table_lists_each_skill_with_its_experience_and_last_use() {
    let xlsx = rendered(&fixture_skill_sheet());

    let (header, _) = xlsx.find("分類").unwrap();
    assert_eq!(xlsx.cell(header, 3).as_deref(), Some("スキル"));
    assert_eq!(xlsx.cell(header, 7).as_deref(), Some("経験"));
    assert_eq!(xlsx.cell(header, 9).as_deref(), Some("最終使用"));
    let rows = skill_rows(&xlsx);
    assert_eq!(rows.len(), 9);
    let first_two: Vec<_> = rows.iter().take(2).map(|row| row[1..].to_vec()).collect();
    assert_eq!(
        first_two,
        [
            [
                Some("タイプスクリプト".to_owned()),
                Some("4年2ヶ月".to_owned()),
                Some("2026年11月".to_owned())
            ],
            [
                Some("Go".to_owned()),
                Some("1年6ヶ月".to_owned()),
                Some("2022年9月".to_owned())
            ],
        ]
    );
}

#[test]
fn a_category_is_named_once_over_all_of_its_skills() {
    let xlsx = rendered(&fixture_skill_sheet());

    let (header, _) = xlsx.find("分類").unwrap();
    assert_eq!(
        xlsx.cell(header + 1, 0).as_deref(),
        Some("プログラミング言語")
    );
    assert!(xlsx.merges().contains(&(header + 1, 0, header + 2, 2)));
    assert_eq!(xlsx.cell(header + 2, 0), None);
}

#[test]
fn the_self_pr_has_one_row_per_paragraph_under_one_label() {
    let xlsx = rendered(&fixture_skill_sheet());

    let (row, column) = xlsx.find("自己PR").expect("the self-PR has a label");
    assert_eq!(
        xlsx.cell(row, column + 2).as_deref(),
        Some(
            "物流とECの分野で7年の経験を持つフルスタック開発者。GoのバックエンドからReactのフロントエンドまで担当する。"
        )
    );
    assert_eq!(
        xlsx.cell(row + 1, column + 2).as_deref(),
        Some("機能の要望からリリースまで一貫して担当する。")
    );
    assert!(xlsx.merges().contains(&(row, 0, row + 1, 1)));
}

#[test]
fn a_project_block_opens_with_its_number_title_and_period() {
    let xlsx = rendered(&fixture_skill_sheet());

    let top = project_top(&xlsx, 2);
    assert_eq!(
        xlsx.cell(top, 1).as_deref(),
        Some("ウェブショップの商品検索")
    );
    assert_eq!(
        xlsx.cell(top, 8).as_deref(),
        Some("2023年10月～現在（3年2ヶ月）")
    );
}

#[test]
fn a_project_block_states_client_industry_contract_role_and_team() {
    let xlsx = rendered(&fixture_skill_sheet());

    let top = project_top(&xlsx, 2);
    let facts = [
        (top + 1, 0, "会社", "Globex"),
        (top + 1, 4, "業種", "小売"),
        (top + 1, 8, "契約", "業務委託契約"),
        (top + 2, 0, "役割", "フルスタック開発者（Search）"),
        (top + 2, 4, "体制", "8名"),
    ];
    for (row, column, label, value) in facts {
        assert_eq!(xlsx.cell(row, column).as_deref(), Some(label));
        assert_eq!(
            xlsx.cell(row, column + 1).as_deref(),
            Some(value),
            "{label}"
        );
    }
}

#[test]
fn a_project_block_names_the_end_client_beside_the_client() {
    let xlsx = rendered(&fixture_skill_sheet());

    let top = project_top(&xlsx, 4);
    assert_eq!(
        xlsx.cell(top + 1, 1).as_deref(),
        Some("株式会社アクメ ／ Contoso")
    );
}

#[test]
fn an_unknown_team_size_prints_the_unknown_mark() {
    let xlsx = rendered(&fixture_skill_sheet());

    assert_eq!(
        xlsx.cell(project_top(&xlsx, 1) + 2, 5).as_deref(),
        Some("—")
    );
}

#[test]
fn the_phase_strip_fills_in_the_phases_the_project_covered() {
    let mut xlsx = rendered(&fixture_skill_sheet());

    let no_phase_done = xlsx.fill_of(project_top(&xlsx, 1) + 3, 0);
    let strip = project_top(&xlsx, 2) + 3;
    let done = xlsx.fill_of(strip, 0);
    assert_ne!(done, no_phase_done);
    let covered: Vec<bool> = (0..6)
        .map(|phase| xlsx.fill_of(strip, phase * 2) == done)
        .collect();
    assert_eq!(covered, [true, true, false, true, false, true]);
    assert_eq!(xlsx.cell(strip, 4).as_deref(), Some("詳細設計"));
}

#[test]
fn a_covered_phase_also_carries_the_done_mark_in_its_cell() {
    let xlsx = rendered(&fixture_skill_sheet());

    let strip = project_top(&xlsx, 2) + 3;
    let phases: Vec<Option<String>> = (0..6).map(|phase| xlsx.cell(strip, phase * 2)).collect();
    let expected = [
        "● 要件定義",
        "● 基本設計",
        "詳細設計",
        "● 製造",
        "テスト",
        "● 運用・保守",
    ];
    assert_eq!(phases, expected.map(|text| Some(text.to_owned())));
}

#[test]
fn a_project_block_lists_its_technology_by_category() {
    let xlsx = rendered(&fixture_skill_sheet());

    let first = project_top(&xlsx, 2) + 4;
    let rows: Vec<(Option<String>, Option<String>)> = (first..first + 5)
        .map(|row| (xlsx.cell(row, 0), xlsx.cell(row, 2)))
        .collect();
    let expected = [
        ("プログラミング言語", "タイプスクリプト"),
        ("フレームワーク", "React"),
        ("クラウド", "Google Cloud"),
        ("プラットフォーム", "Node.js"),
        ("AIツール", "Claude Code"),
    ];
    assert_eq!(
        rows,
        expected.map(|(category, skills)| (Some(category.to_owned()), Some(skills.to_owned())))
    );
}

#[test]
fn a_project_block_ends_with_its_description_and_achievements() {
    let xlsx = rendered(&fixture_skill_sheet());

    let (top, next) = (project_top(&xlsx, 2), project_top(&xlsx, 3));
    let texts = texts_between(&xlsx, top + 9, next);
    assert_eq!(
        texts,
        [
            "【業務内容】",
            "Develops the product search of the web shop, from the ranking service on Google Cloud to the React front end.",
            "【実績】",
            "・Introduces typo-tolerant search",
        ]
    );
}

#[test]
fn a_project_without_achievements_has_no_achievements_heading() {
    let xlsx = rendered(&fixture_skill_sheet());

    let texts = texts_between(&xlsx, project_top(&xlsx, 1), project_top(&xlsx, 2));
    assert!(texts.contains(&"【業務内容】".to_owned()));
    assert!(!texts.contains(&"【実績】".to_owned()));
}

#[test]
fn the_project_blocks_follow_in_the_models_order() {
    let xlsx = rendered(&fixture_skill_sheet());

    let tops: Vec<u32> = (1..=5).map(|number| project_top(&xlsx, number)).collect();
    assert!(tops.is_sorted());
}

#[test]
fn every_cell_is_set_in_biz_udp_gothic() {
    let mut xlsx = rendered(&fixture_skill_sheet());

    let cells = xlsx.cells();
    assert!(cells.len() > 20, "{} cells", cells.len());
    for (row, column, text) in cells {
        assert_eq!(
            xlsx.font_name_of(row, column).as_deref(),
            Some("BIZ UDPゴシック"),
            "{text}"
        );
    }
}

#[test]
fn the_sheet_has_cells_only_and_no_image_or_drawing() {
    let xlsx = rendered(&fixture_skill_sheet());

    let parts = xlsx.part_names();
    assert!(
        parts
            .iter()
            .all(|part| !part.starts_with("xl/media") && !part.starts_with("xl/drawings")),
        "{parts:?}"
    );
}

#[test]
fn the_sheet_prints_on_a_portrait_a4_with_page_numbers() {
    let mut xlsx = rendered(&fixture_skill_sheet());

    assert!(
        xlsx.sheet
            .contains("paperSize=\"9\" orientation=\"portrait\"")
    );
    assert!(
        xlsx.sheet
            .contains("<oddFooter>&amp;C&amp;8&amp;P / &amp;N</oddFooter>")
    );
    let workbook = xlsx.part("xl/workbook.xml");
    assert!(workbook.contains("Print_Area"), "{workbook}");
}

#[test]
fn the_grid_is_narrower_than_the_printable_width_of_an_a4_page() {
    let xlsx = rendered(&fixture_skill_sheet());

    // A stored width is the column's pixels over 7, its padding included, as Excel lays out the default font.
    let grid_pixels: f64 = xlsx.column_widths().values().map(|width| width * 7.0).sum();
    // 210 mm less 2 × 0.4 inch of margin, at 96 pixels to the inch.
    let printable_pixels = (210.0 / 25.4 - 0.8) * 96.0;
    assert!(
        grid_pixels <= printable_pixels,
        "{grid_pixels} > {printable_pixels}"
    );
    assert!(!xlsx.sheet.contains("fitToPage"));
}

#[test]
fn a_long_paragraph_gets_a_row_as_tall_as_its_lines() {
    let mut sheet = fixture_skill_sheet();
    sheet.self_pr = vec!["あ".repeat(400), "短い段落".to_owned()];

    let xlsx = rendered(&sheet);

    let (row, _) = xlsx.find("自己PR").unwrap();
    let heights = xlsx.row_heights();
    assert!(
        (100.0..=140.0).contains(&heights[&row]),
        "{}",
        heights[&row]
    );
    assert!(heights[&(row + 1)] < 25.0, "{}", heights[&(row + 1)]);
}

#[test]
fn no_row_is_taller_than_excel_allows() {
    let mut sheet = fixture_skill_sheet();
    sheet.self_pr = vec!["あ".repeat(20_000)];

    let xlsx = rendered(&sheet);

    let (row, _) = xlsx.find("自己PR").unwrap();
    let heights = xlsx.row_heights();
    assert!(heights[&row] > 400.0, "{}", heights[&row]);
    assert!(heights.values().all(|height| *height <= 409.0));
}

#[test]
fn page_breaks_come_only_before_a_project_block() {
    let mut sheet = fixture_skill_sheet();
    sheet.projects = sheet.projects.iter().cycle().take(15).cloned().collect();

    let xlsx = rendered(&sheet);

    let breaks = xlsx.page_breaks();
    assert!(breaks.len() >= 2, "{breaks:?}");
    for row in breaks {
        let first_cell = xlsx.cell(row, 0).unwrap_or_default();
        assert!(
            first_cell.starts_with("No. "),
            "row {row} starts with {first_cell:?}"
        );
    }
}

#[test]
fn the_document_properties_are_fixed_so_the_file_is_reproducible() {
    let sheet = fixture_skill_sheet();
    let mut xlsx = rendered(&sheet);

    let properties = xlsx.part("docProps/core.xml");
    assert!(
        properties.contains("<dc:title>スキルシート</dc:title>"),
        "{properties}"
    );
    assert!(properties.contains("<dc:creator>Robin Beispiel</dc:creator>"));
    assert_eq!(
        properties.matches("2000-01-01T00:00:00Z").count(),
        2,
        "{properties}"
    );
    assert_eq!(render(&sheet).unwrap(), render(&sheet).unwrap());
}
