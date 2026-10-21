use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use sqlx::SqlitePool;

use super::queries::{
    ExportAlignment, ExportCell, ExportDataset, ExportFilters, ExportKind,
    finalize_export_destination, load_export_dataset, prepare_export_destination,
};
use crate::error::CommandError;

/// Exports the requested report as a printable, high-contrast light-themed `.xlsx` spreadsheet.
/// Writes to a temporary file first and atomically renames to `dest_path`.
pub async fn export_xlsx(
    pool: &SqlitePool,
    kind: ExportKind,
    filters: &ExportFilters,
    dest_path: &str,
) -> Result<String, CommandError> {
    let (final_path, temp_path) = prepare_export_destination(dest_path)?;
    let dataset = load_export_dataset(pool, kind, filters).await?;

    if let Err(error) = build_workbook(&dataset, &temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        eprintln!("XLSX export error: {error}");
        return Err(CommandError::new(
            "export_failed",
            "No se pudo generar la planilla XLSX.",
        ));
    }

    finalize_export_destination(&final_path, &temp_path)
}

fn to_format_align(align: ExportAlignment) -> FormatAlign {
    match align {
        ExportAlignment::Left => FormatAlign::Left,
        ExportAlignment::Center => FormatAlign::Center,
        ExportAlignment::Right => FormatAlign::Right,
    }
}

fn build_workbook(dataset: &ExportDataset, path: &std::path::Path) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let sheet_name = match dataset.kind {
        ExportKind::DailySheet => "Planilla diaria",
        ExportKind::ServicesByStaff => "Servicios por persona",
        ExportKind::AverageTimes => "Tiempos promedio",
        ExportKind::AbsencesByMonth => "Ausencias por mes",
    };
    worksheet.set_name(sheet_name)?;

    // Page setup for A4 landscape printable output
    worksheet.set_paper_size(9); // A4
    worksheet.set_landscape();
    worksheet.set_print_fit_to_pages(1, 0);
    worksheet.set_print_center_horizontally(true);
    worksheet.set_margins(0.4, 0.4, 0.6, 0.6, 0.25, 0.25);
    worksheet.set_header("&LGestión de Limpieza&R&D");
    worksheet.set_footer("&LDocumento imprimible&RPágina &P de &N");

    let last_col = (dataset.columns.len().saturating_sub(1)) as u16;

    for (idx, col) in dataset.columns.iter().enumerate() {
        worksheet.set_column_width(idx as u16, col.width_chars)?;
    }

    let title_format = Format::new()
        .set_bold()
        .set_font_size(15.0)
        .set_font_color(Color::RGB(0x0F172A))
        .set_background_color(Color::RGB(0xF1F5F9))
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(0xCBD5E1));

    let subtitle_format = Format::new()
        .set_font_size(10.5)
        .set_font_color(Color::RGB(0x334155))
        .set_background_color(Color::RGB(0xF8FAFC))
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(0xCBD5E1));

    worksheet.set_row_height(0, 26.0)?;
    worksheet.set_row_height(1, 20.0)?;
    worksheet.set_row_height(2, 8.0)?;
    worksheet.set_row_height(3, 24.0)?;

    if last_col > 0 {
        worksheet.merge_range(0, 0, 0, last_col, &dataset.title, &title_format)?;
        worksheet.merge_range(1, 0, 1, last_col, &dataset.subtitle, &subtitle_format)?;
    } else {
        worksheet.write_with_format(0, 0, &dataset.title, &title_format)?;
        worksheet.write_with_format(1, 0, &dataset.subtitle, &subtitle_format)?;
    }

    // Repeat title, subtitle and header row on every printed page
    worksheet.set_repeat_rows(0, 3)?;

    // Header row at row index 3
    for (col_idx, col) in dataset.columns.iter().enumerate() {
        let header_format = Format::new()
            .set_bold()
            .set_font_size(10.5)
            .set_font_color(Color::RGB(0xFFFFFF))
            .set_background_color(Color::RGB(0x1E293B))
            .set_align(to_format_align(col.align))
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0x0F172A))
            .set_text_wrap();
        worksheet.write_with_format(3, col_idx as u16, col.header, &header_format)?;
    }

    let mut current_row: u32 = 4;

    if dataset.rows.is_empty() {
        worksheet.set_row_height(current_row, 22.0)?;
        let empty_format = Format::new()
            .set_italic()
            .set_font_size(10.0)
            .set_font_color(Color::RGB(0x475569))
            .set_background_color(Color::RGB(0xFFFFFF))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xCBD5E1));
        if last_col > 0 {
            worksheet.merge_range(
                current_row,
                0,
                current_row,
                last_col,
                "Sin registros para el filtro seleccionado.",
                &empty_format,
            )?;
        } else {
            worksheet.write_with_format(
                current_row,
                0,
                "Sin registros para el filtro seleccionado.",
                &empty_format,
            )?;
        }
        current_row += 1;
    } else {
        for (row_idx, row) in dataset.rows.iter().enumerate() {
            worksheet.set_row_height(current_row, 20.0)?;
            let bg_color = if row_idx % 2 == 0 {
                Color::RGB(0xFFFFFF)
            } else {
                Color::RGB(0xF8FAFC)
            };

            for (col_idx, cell) in row.iter().enumerate() {
                let align = dataset
                    .columns
                    .get(col_idx)
                    .map(|c| c.align)
                    .unwrap_or(ExportAlignment::Left);
                let base_format = Format::new()
                    .set_font_size(10.0)
                    .set_font_color(Color::RGB(0x0F172A))
                    .set_background_color(bg_color)
                    .set_align(to_format_align(align))
                    .set_align(FormatAlign::VerticalCenter)
                    .set_border(FormatBorder::Thin)
                    .set_border_color(Color::RGB(0xCBD5E1))
                    .set_text_wrap();

                write_cell(worksheet, current_row, col_idx as u16, cell, base_format)?;
            }
            current_row += 1;
        }
    }

    // Totals row
    worksheet.set_row_height(current_row, 22.0)?;
    for (col_idx, cell) in dataset.totals_row.iter().enumerate() {
        let align = dataset
            .columns
            .get(col_idx)
            .map(|c| c.align)
            .unwrap_or(ExportAlignment::Left);
        let totals_format = Format::new()
            .set_bold()
            .set_font_size(10.0)
            .set_font_color(Color::RGB(0x0F172A))
            .set_background_color(Color::RGB(0xE2E8F0))
            .set_align(to_format_align(align))
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_border_top(FormatBorder::Medium)
            .set_border_color(Color::RGB(0x64748B));

        write_cell(worksheet, current_row, col_idx as u16, cell, totals_format)?;
    }

    workbook.save(path)?;
    Ok(())
}

fn write_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    cell: &ExportCell,
    format: Format,
) -> Result<(), XlsxError> {
    match cell {
        ExportCell::Text(text) => {
            worksheet.write_string_with_format(row, col, text, &format)?;
        }
        ExportCell::Int(num) => {
            let fmt = format.set_num_format("#,##0");
            worksheet.write_number_with_format(row, col, *num as f64, &fmt)?;
        }
        ExportCell::Float1(num) => {
            let fmt = format.set_num_format("0.0");
            worksheet.write_number_with_format(row, col, *num, &fmt)?;
        }
        ExportCell::Float2(num) => {
            let fmt = format.set_num_format("0.00");
            worksheet.write_number_with_format(row, col, *num, &fmt)?;
        }
    }
    Ok(())
}
