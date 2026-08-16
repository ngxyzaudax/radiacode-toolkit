use crate::ui::table::{ColumnAlign, TableColumn, TableLayout};

pub fn nuclide_table_layout(width: f32) -> TableLayout {
    let lines_w = 36.0;
    let half_life_w = 88.0;
    let name_w = (width - lines_w - half_life_w - 16.0 - 16.0).max(64.0);
    let row_width = 16.0 + name_w + 8.0 + half_life_w + 8.0 + lines_w;
    TableLayout {
        columns: vec![
            TableColumn {
                label: "Name",
                width: name_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "Half-life",
                width: half_life_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "γ",
                width: lines_w,
                align: ColumnAlign::Right,
            },
        ],
        row_width,
    }
}

pub fn contributor_table_layout(width: f32) -> TableLayout {
    let half_life_w = 88.0;
    let share_w = 80.0;
    let fixed = half_life_w + share_w + 16.0 + 16.0;
    let name_w = (width - fixed).max(72.0);
    let row_width = 16.0 + name_w + 8.0 + half_life_w + 8.0 + share_w;
    TableLayout {
        columns: vec![
            TableColumn {
                label: "Nuclide",
                width: name_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "Half-life",
                width: half_life_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "Share",
                width: share_w,
                align: ColumnAlign::Left,
            },
        ],
        row_width,
    }
}

pub fn radiation_tree_layout(width: f32) -> TableLayout {
    let type_w = 24.0;
    let intensity_w = 72.0;
    let fixed = type_w + intensity_w + 16.0 + 16.0;
    let energy_w = (width - fixed).max(52.0);
    let row_width = energy_w + fixed;
    TableLayout {
        columns: vec![
            TableColumn {
                label: "Type",
                width: type_w,
                align: ColumnAlign::Center,
            },
            TableColumn {
                label: "Energy",
                width: energy_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "I",
                width: intensity_w,
                align: ColumnAlign::Left,
            },
        ],
        row_width,
    }
}
