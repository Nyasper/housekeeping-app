use printpdf::{
    Color, FontId, Line, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle,
    PdfPage, PdfSaveOptions, Point, Pt, Rect, Rgb, TextItem, WindingOrder,
};
use sqlx::SqlitePool;

use super::queries::{
    ExportAlignment, ExportColumn, ExportDataset, ExportFilters, ExportKind,
    finalize_export_destination, load_export_dataset, prepare_export_destination,
};
use crate::error::CommandError;

const DEJAVU_SANS_TTF: &[u8] = include_bytes!("../../../assets/fonts/DejaVuSans.ttf");

const PAGE_WIDTH_MM: f32 = 297.0;
const PAGE_HEIGHT_MM: f32 = 210.0;
const MARGIN_X_MM: f32 = 14.0;
const MARGIN_TOP_MM: f32 = 14.0;
const MARGIN_BOTTOM_MM: f32 = 14.0;
const FOOTER_RESERVE_MM: f32 = 10.0;
const HEADER_ROW_HEIGHT_MM: f32 = 8.5;
const LINE_HEIGHT_MM: f32 = 4.2;
const ROW_PADDING_Y_MM: f32 = 2.6;
const CELL_PADDING_X_MM: f32 = 2.0;
/// Keep a single row below the usable table height so it cannot overlap the
/// footer when it is the first row of a page.
const MAX_ROW_HEIGHT_MM: f32 = 140.0;

/// Exports the requested report as a printable A4 landscape PDF document with embedded
/// `DejaVuSans` font, automatic page breaks, and page numbering.
pub async fn export_pdf(
    pool: &SqlitePool,
    kind: ExportKind,
    filters: &ExportFilters,
    dest_path: &str,
) -> Result<String, CommandError> {
    let (final_path, temp_path) = prepare_export_destination(dest_path)?;
    let dataset = load_export_dataset(pool, kind, filters).await?;

    let bytes = render_pdf_bytes(&dataset)?;
    if let Err(error) = std::fs::write(&temp_path, bytes) {
        let _ = std::fs::remove_file(&temp_path);
        eprintln!("PDF write error: {error}");
        return Err(CommandError::new(
            "export_failed",
            "No se pudo escribir el archivo PDF en la ruta seleccionada.",
        ));
    }

    finalize_export_destination(&final_path, &temp_path)
}

fn rgb_hex(hex: u32) -> Color {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    Color::Rgb(Rgb::new(r, g, b, None))
}

fn pt_from_mm(mm: f32) -> Pt {
    Mm(mm).into()
}

fn point_mm(x_mm: f32, y_mm: f32) -> Point {
    Point {
        x: pt_from_mm(x_mm),
        y: pt_from_mm(y_mm),
    }
}

fn draw_rect_ops(
    ops: &mut Vec<Op>,
    x_mm: f32,
    y_mm: f32,
    width_mm: f32,
    height_mm: f32,
    fill: Option<Color>,
    stroke: Option<(Color, f32)>,
) {
    let mode = match (&fill, &stroke) {
        (Some(_), Some(_)) => PaintMode::FillStroke,
        (Some(_), None) => PaintMode::Fill,
        (None, Some(_)) => PaintMode::Stroke,
        (None, None) => return,
    };

    if let Some(fill_color) = fill {
        ops.push(Op::SetFillColor { col: fill_color });
    }
    if let Some((stroke_color, thickness_pt)) = stroke {
        ops.push(Op::SetOutlineColor { col: stroke_color });
        ops.push(Op::SetOutlineThickness {
            pt: Pt(thickness_pt),
        });
    }

    ops.push(Op::DrawRectangle {
        rectangle: Rect {
            x: pt_from_mm(x_mm),
            y: pt_from_mm(y_mm),
            width: pt_from_mm(width_mm),
            height: pt_from_mm(height_mm),
            mode: Some(mode),
            winding_order: Some(WindingOrder::NonZero),
        },
    });
}

fn draw_line_ops(
    ops: &mut Vec<Op>,
    x1_mm: f32,
    y1_mm: f32,
    x2_mm: f32,
    y2_mm: f32,
    color: Color,
    thickness_pt: f32,
) {
    ops.push(Op::SetOutlineColor { col: color });
    ops.push(Op::SetOutlineThickness {
        pt: Pt(thickness_pt),
    });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: point_mm(x1_mm, y1_mm),
                    bezier: false,
                },
                LinePoint {
                    p: point_mm(x2_mm, y2_mm),
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}

fn draw_text_ops(
    ops: &mut Vec<Op>,
    font_id: &FontId,
    font_size_pt: f32,
    x_mm: f32,
    y_mm: f32,
    color: Color,
    text: &str,
) {
    if text.is_empty() {
        return;
    }
    ops.push(Op::SetFillColor { col: color });
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font_id.clone()),
        size: Pt(font_size_pt),
    });
    ops.push(Op::SetTextCursor {
        pos: point_mm(x_mm, y_mm),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text.to_string())],
    });
    ops.push(Op::EndTextSection);
}

fn estimate_text_width_mm(text: &str, font_size_pt: f32) -> f32 {
    // 1 pt = 0.352778 mm; DejaVuSans average character width is ~0.54 em
    let pt_to_mm = 0.352_778_f32;
    let width_em: f32 = text
        .chars()
        .map(|ch| match ch {
            ' ' | '.' | ',' | ':' | ';' | 'i' | 'l' | 'I' | '!' | '|' => 0.30,
            'm' | 'w' | 'M' | 'W' | '@' | '%' => 0.78,
            c if c.is_uppercase() => 0.64,
            _ => 0.54,
        })
        .sum();
    width_em * font_size_pt * pt_to_mm
}

fn wrap_text_lines(text: &str, max_width_mm: f32, font_size_pt: f32) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return vec![String::new()];
    }

    let usable = (max_width_mm - 2.0 * CELL_PADDING_X_MM).max(8.0);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in trimmed.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };

        if estimate_text_width_mm(&candidate, font_size_pt) <= usable {
            current = candidate;
        } else {
            if !current.is_empty() {
                lines.push(current);
            }
            // If a single word exceeds the column width, split it by characters
            if estimate_text_width_mm(word, font_size_pt) > usable {
                let mut chunk = String::new();
                for ch in word.chars() {
                    let next = format!("{chunk}{ch}");
                    if estimate_text_width_mm(&next, font_size_pt) > usable && !chunk.is_empty() {
                        lines.push(chunk);
                        chunk = ch.to_string();
                    } else {
                        chunk = next;
                    }
                }
                current = chunk;
            } else {
                current = word.to_string();
            }
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn ellipsize_to_width(text: &str, max_width_mm: f32, font_size_pt: f32) -> String {
    let usable = (max_width_mm - 2.0 * CELL_PADDING_X_MM - 0.5).max(8.0);
    let ellipsis = "…";
    let mut chars: Vec<char> = text.chars().collect();
    loop {
        let candidate = format!("{}{ellipsis}", chars.iter().collect::<String>());
        if estimate_text_width_mm(&candidate, font_size_pt) <= usable || chars.is_empty() {
            return candidate;
        }
        chars.pop();
    }
}

/// Truncates the tallest cells so a single row always fits on one page.
fn cap_wrapped_row(
    wrapped_cells: &mut [Vec<String>],
    col_widths: &[f32],
    font_size_pt: f32,
    padding_y_mm: f32,
) -> f32 {
    let max_lines = (((MAX_ROW_HEIGHT_MM - padding_y_mm) / LINE_HEIGHT_MM).floor() as usize).max(1);
    for (idx, lines) in wrapped_cells.iter_mut().enumerate() {
        if lines.len() > max_lines {
            lines.truncate(max_lines);
            if let Some(last) = lines.last_mut() {
                let col_w = col_widths.get(idx).copied().unwrap_or(30.0);
                *last = ellipsize_to_width(last, col_w, font_size_pt);
            }
        }
    }
    let max_lines_in_row = wrapped_cells.iter().map(Vec::len).max().unwrap_or(1).max(1);
    (max_lines_in_row as f32) * LINE_HEIGHT_MM + padding_y_mm
}

fn normalized_column_widths(columns: &[ExportColumn]) -> Vec<f32> {
    let usable_width = PAGE_WIDTH_MM - 2.0 * MARGIN_X_MM;
    let raw_sum: f32 = columns.iter().map(|c| c.width_mm).sum();
    if raw_sum <= 0.0 {
        let equal = usable_width / (columns.len().max(1) as f32);
        return vec![equal; columns.len()];
    }
    columns
        .iter()
        .map(|c| (c.width_mm / raw_sum) * usable_width)
        .collect()
}

struct PreparedRow {
    wrapped_cells: Vec<Vec<String>>,
    height_mm: f32,
    is_totals: bool,
    is_empty_placeholder: bool,
}

fn prepare_table_rows(dataset: &ExportDataset, col_widths: &[f32]) -> Vec<PreparedRow> {
    let mut prepared = Vec::new();

    if dataset.rows.is_empty() {
        let mut cells = vec![vec![String::new()]; dataset.columns.len()];
        if let Some(first) = cells.first_mut() {
            *first = vec!["Sin registros para el filtro seleccionado.".to_string()];
        }
        prepared.push(PreparedRow {
            wrapped_cells: cells,
            height_mm: LINE_HEIGHT_MM + ROW_PADDING_Y_MM * 2.0,
            is_totals: false,
            is_empty_placeholder: true,
        });
    } else {
        for row in &dataset.rows {
            let mut wrapped_cells: Vec<Vec<String>> = row
                .iter()
                .enumerate()
                .map(|(idx, cell)| {
                    let col_w = col_widths.get(idx).copied().unwrap_or(30.0);
                    wrap_text_lines(&cell.to_display_es(), col_w, 8.5)
                })
                .collect();
            let max_lines = wrapped_cells
                .iter()
                .map(|lines| lines.len())
                .max()
                .unwrap_or(1)
                .max(1);
            let mut height_mm = (max_lines as f32) * LINE_HEIGHT_MM + ROW_PADDING_Y_MM * 2.0;
            if height_mm > MAX_ROW_HEIGHT_MM {
                height_mm =
                    cap_wrapped_row(&mut wrapped_cells, col_widths, 8.5, ROW_PADDING_Y_MM * 2.0);
            }
            prepared.push(PreparedRow {
                wrapped_cells,
                height_mm,
                is_totals: false,
                is_empty_placeholder: false,
            });
        }
    }

    let mut totals_wrapped: Vec<Vec<String>> = dataset
        .totals_row
        .iter()
        .enumerate()
        .map(|(idx, cell)| {
            let col_w = col_widths.get(idx).copied().unwrap_or(30.0);
            wrap_text_lines(&cell.to_display_es(), col_w, 8.8)
        })
        .collect();
    let max_lines = totals_wrapped
        .iter()
        .map(|lines| lines.len())
        .max()
        .unwrap_or(1)
        .max(1);
    let mut totals_height = (max_lines as f32) * LINE_HEIGHT_MM + ROW_PADDING_Y_MM * 2.2;
    if totals_height > MAX_ROW_HEIGHT_MM {
        totals_height =
            cap_wrapped_row(&mut totals_wrapped, col_widths, 8.8, ROW_PADDING_Y_MM * 2.2);
    }
    prepared.push(PreparedRow {
        wrapped_cells: totals_wrapped,
        height_mm: totals_height,
        is_totals: true,
        is_empty_placeholder: false,
    });

    prepared
}

fn draw_page_header_and_columns(
    ops: &mut Vec<Op>,
    font_id: &FontId,
    dataset: &ExportDataset,
    col_widths: &[f32],
) -> f32 {
    let usable_width = PAGE_WIDTH_MM - 2.0 * MARGIN_X_MM;
    let top_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM;

    // Header banner box
    let banner_height = 16.0;
    let banner_bottom = top_y - banner_height;
    draw_rect_ops(
        ops,
        MARGIN_X_MM,
        banner_bottom,
        usable_width,
        banner_height,
        Some(rgb_hex(0xF8FAFC)),
        Some((rgb_hex(0xCBD5E1), 0.6)),
    );

    // Title & subtitle
    draw_text_ops(
        ops,
        font_id,
        13.5,
        MARGIN_X_MM + 3.5,
        top_y - 6.2,
        rgb_hex(0x0F172A),
        &dataset.title,
    );
    draw_text_ops(
        ops,
        font_id,
        9.0,
        MARGIN_X_MM + 3.5,
        top_y - 12.2,
        rgb_hex(0x334155),
        &dataset.subtitle,
    );

    let app_tag = "Gestión de Limpieza";
    let tag_width = estimate_text_width_mm(app_tag, 8.5);
    draw_text_ops(
        ops,
        font_id,
        8.5,
        PAGE_WIDTH_MM - MARGIN_X_MM - tag_width - 3.5,
        top_y - 6.2,
        rgb_hex(0x475569),
        app_tag,
    );

    // Table column headers
    let header_top = banner_bottom - 3.0;
    let header_bottom = header_top - HEADER_ROW_HEIGHT_MM;

    let mut cur_x = MARGIN_X_MM;
    for (col_idx, col) in dataset.columns.iter().enumerate() {
        let col_w = col_widths.get(col_idx).copied().unwrap_or(25.0);
        draw_rect_ops(
            ops,
            cur_x,
            header_bottom,
            col_w,
            HEADER_ROW_HEIGHT_MM,
            Some(rgb_hex(0x1E293B)),
            Some((rgb_hex(0x0F172A), 0.5)),
        );

        let text_w = estimate_text_width_mm(col.header, 8.5);
        let text_x = match col.align {
            ExportAlignment::Left => cur_x + CELL_PADDING_X_MM,
            ExportAlignment::Center => cur_x + ((col_w - text_w) / 2.0).max(CELL_PADDING_X_MM),
            ExportAlignment::Right => {
                (cur_x + col_w - CELL_PADDING_X_MM - text_w).max(cur_x + CELL_PADDING_X_MM)
            }
        };
        let text_y = header_bottom + 2.8;
        draw_text_ops(
            ops,
            font_id,
            8.5,
            text_x,
            text_y,
            rgb_hex(0xFFFFFF),
            col.header,
        );

        cur_x += col_w;
    }

    header_bottom
}

fn draw_prepared_row(
    ops: &mut Vec<Op>,
    font_id: &FontId,
    dataset: &ExportDataset,
    col_widths: &[f32],
    row: &PreparedRow,
    row_index: usize,
    top_y: f32,
) {
    let bottom_y = top_y - row.height_mm;
    let usable_width = PAGE_WIDTH_MM - 2.0 * MARGIN_X_MM;

    if row.is_empty_placeholder {
        draw_rect_ops(
            ops,
            MARGIN_X_MM,
            bottom_y,
            usable_width,
            row.height_mm,
            Some(rgb_hex(0xFFFFFF)),
            Some((rgb_hex(0xCBD5E1), 0.45)),
        );
        let msg = "Sin registros para el filtro seleccionado.";
        let msg_w = estimate_text_width_mm(msg, 8.5);
        let text_x = MARGIN_X_MM + ((usable_width - msg_w) / 2.0).max(CELL_PADDING_X_MM);
        let text_y = top_y - ROW_PADDING_Y_MM - 3.0;
        draw_text_ops(ops, font_id, 8.5, text_x, text_y, rgb_hex(0x475569), msg);
        return;
    }

    let bg_color = if row.is_totals {
        rgb_hex(0xE2E8F0)
    } else if row_index.is_multiple_of(2) {
        rgb_hex(0xFFFFFF)
    } else {
        rgb_hex(0xF8FAFC)
    };

    let border_color = if row.is_totals {
        rgb_hex(0x64748B)
    } else {
        rgb_hex(0xCBD5E1)
    };

    let font_size = if row.is_totals { 8.8 } else { 8.4 };

    let mut cur_x = MARGIN_X_MM;
    for (col_idx, lines) in row.wrapped_cells.iter().enumerate() {
        let col_w = col_widths.get(col_idx).copied().unwrap_or(25.0);
        let align = dataset
            .columns
            .get(col_idx)
            .map(|c| c.align)
            .unwrap_or(ExportAlignment::Left);

        draw_rect_ops(
            ops,
            cur_x,
            bottom_y,
            col_w,
            row.height_mm,
            Some(bg_color.clone()),
            Some((border_color.clone(), 0.45)),
        );

        for (line_idx, line_text) in lines.iter().enumerate() {
            if line_text.is_empty() {
                continue;
            }
            let text_w = estimate_text_width_mm(line_text, font_size);
            let text_x = match align {
                ExportAlignment::Left => cur_x + CELL_PADDING_X_MM,
                ExportAlignment::Center => cur_x + ((col_w - text_w) / 2.0).max(CELL_PADDING_X_MM),
                ExportAlignment::Right => {
                    (cur_x + col_w - CELL_PADDING_X_MM - text_w).max(cur_x + CELL_PADDING_X_MM)
                }
            };
            let text_y = top_y - ROW_PADDING_Y_MM - 3.0 - (line_idx as f32) * LINE_HEIGHT_MM;
            draw_text_ops(
                ops,
                font_id,
                font_size,
                text_x,
                text_y,
                rgb_hex(0x0F172A),
                line_text,
            );
        }

        cur_x += col_w;
    }

    if row.is_totals {
        draw_line_ops(
            ops,
            MARGIN_X_MM,
            top_y,
            MARGIN_X_MM + usable_width,
            top_y,
            rgb_hex(0x334155),
            0.9,
        );
    }
}

fn draw_page_footer(ops: &mut Vec<Op>, font_id: &FontId, page_num: usize, total_pages: usize) {
    let footer_line_y = MARGIN_BOTTOM_MM + 5.5;
    draw_line_ops(
        ops,
        MARGIN_X_MM,
        footer_line_y,
        PAGE_WIDTH_MM - MARGIN_X_MM,
        footer_line_y,
        rgb_hex(0xCBD5E1),
        0.5,
    );

    let left_note = "Planilla imprimible · Gestión de Limpieza";
    draw_text_ops(
        ops,
        font_id,
        8.0,
        MARGIN_X_MM,
        MARGIN_BOTTOM_MM + 1.5,
        rgb_hex(0x475569),
        left_note,
    );

    let page_label = format!("Página {page_num} de {total_pages}");
    let label_w = estimate_text_width_mm(&page_label, 8.0);
    draw_text_ops(
        ops,
        font_id,
        8.0,
        PAGE_WIDTH_MM - MARGIN_X_MM - label_w,
        MARGIN_BOTTOM_MM + 1.5,
        rgb_hex(0x334155),
        &page_label,
    );
}

pub fn render_pdf_bytes(dataset: &ExportDataset) -> Result<Vec<u8>, CommandError> {
    let mut warnings = Vec::new();
    let parsed_font =
        ParsedFont::from_bytes(DEJAVU_SANS_TTF, 0, &mut warnings).ok_or_else(|| {
            CommandError::new(
                "export_failed",
                "No se pudo cargar la fuente embebida para generar el PDF.",
            )
        })?;

    let mut doc = PdfDocument::new(&dataset.title);
    let font_id = doc.add_font(&parsed_font);

    let col_widths = normalized_column_widths(&dataset.columns);
    let prepared_rows = prepare_table_rows(dataset, &col_widths);

    let min_table_y = MARGIN_BOTTOM_MM + FOOTER_RESERVE_MM;

    // Group rows into pages
    let mut pages_rows: Vec<Vec<(usize, &PreparedRow)>> = Vec::new();
    let mut current_page: Vec<(usize, &PreparedRow)> = Vec::new();
    let initial_cursor_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM - 16.0 - 3.0 - HEADER_ROW_HEIGHT_MM;
    let mut cursor_y = initial_cursor_y;

    for (idx, row) in prepared_rows.iter().enumerate() {
        if !current_page.is_empty() && (cursor_y - row.height_mm) < min_table_y {
            pages_rows.push(current_page);
            current_page = Vec::new();
            cursor_y = initial_cursor_y;
        }
        cursor_y -= row.height_mm;
        current_page.push((idx, row));
    }

    if !current_page.is_empty() || pages_rows.is_empty() {
        pages_rows.push(current_page);
    }

    let total_pages = pages_rows.len();
    let mut pdf_pages = Vec::with_capacity(total_pages);

    for (page_idx, page_rows) in pages_rows.into_iter().enumerate() {
        let mut ops = Vec::new();
        let mut row_top_y = draw_page_header_and_columns(&mut ops, &font_id, dataset, &col_widths);

        for (row_idx, row) in page_rows {
            draw_prepared_row(
                &mut ops,
                &font_id,
                dataset,
                &col_widths,
                row,
                row_idx,
                row_top_y,
            );
            row_top_y -= row.height_mm;
        }

        draw_page_footer(&mut ops, &font_id, page_idx + 1, total_pages);
        pdf_pages.push(PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), ops));
    }

    doc.with_pages(pdf_pages);
    let save_options = PdfSaveOptions::default();
    let mut save_warnings = Vec::new();
    let bytes = doc.save(&save_options, &mut save_warnings);

    if bytes.is_empty() {
        return Err(CommandError::new(
            "export_failed",
            "No se pudo serializar el documento PDF.",
        ));
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::super::queries::ExportCell;
    use super::*;

    #[test]
    fn prepared_rows_are_capped_to_fit_one_page() {
        let dataset = ExportDataset {
            kind: ExportKind::DailySheet,
            title: "Planilla diaria".to_string(),
            subtitle: "Prueba".to_string(),
            columns: vec![ExportColumn {
                header: "Notas",
                width_chars: 60.0,
                width_mm: 260.0,
                align: ExportAlignment::Left,
            }],
            rows: vec![vec![ExportCell::Text("nota ".repeat(2000))]],
            totals_row: vec![ExportCell::Text("Total".to_string())],
        };

        let col_widths = normalized_column_widths(&dataset.columns);
        let rows = prepare_table_rows(&dataset, &col_widths);

        assert!(rows.iter().all(|row| row.height_mm <= MAX_ROW_HEIGHT_MM));
        assert!(
            rows[0].wrapped_cells[0]
                .last()
                .is_some_and(|line| line.ends_with('…'))
        );

        let initial_cursor_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM - 16.0 - 3.0 - HEADER_ROW_HEIGHT_MM;
        let min_table_y = MARGIN_BOTTOM_MM + FOOTER_RESERVE_MM;
        assert!(initial_cursor_y - rows[0].height_mm >= min_table_y);
    }
}
