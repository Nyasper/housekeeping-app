use std::fs::File;
use std::io::Write;

use sqlx::SqlitePool;

use super::queries::{
    ExportFilters, ExportKind, finalize_export_destination, load_export_dataset,
    prepare_export_destination,
};
use crate::error::CommandError;

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// Exports the requested report as a tabular UTF-8 CSV file with a BOM for Excel.
/// Writes to a temporary file first and atomically renames to `dest_path`.
pub async fn export_csv(
    pool: &SqlitePool,
    kind: ExportKind,
    filters: &ExportFilters,
    dest_path: &str,
) -> Result<String, CommandError> {
    let (final_path, temp_path) = prepare_export_destination(dest_path)?;
    let dataset = load_export_dataset(pool, kind, filters).await?;

    let mut file = File::create(&temp_path).map_err(|error| {
        eprintln!("CSV file create error: {error}");
        CommandError::new(
            "export_failed",
            "No se pudo crear el archivo CSV en la ruta seleccionada.",
        )
    })?;

    if let Err(error) = file.write_all(&UTF8_BOM) {
        let _ = std::fs::remove_file(&temp_path);
        eprintln!("CSV BOM write error: {error}");
        return Err(CommandError::new(
            "export_failed",
            "No se pudo escribir el encabezado del archivo CSV.",
        ));
    }

    let mut writer = csv::Writer::from_writer(file);

    let headers: Vec<&str> = dataset.columns.iter().map(|col| col.header).collect();
    if let Err(error) = writer.write_record(&headers) {
        let _ = std::fs::remove_file(&temp_path);
        eprintln!("CSV header write error: {error}");
        return Err(CommandError::new(
            "export_failed",
            "No se pudieron escribir las columnas del archivo CSV.",
        ));
    }

    for row in &dataset.rows {
        let record: Vec<String> = row.iter().map(|cell| cell.to_csv_string()).collect();
        if let Err(error) = writer.write_record(&record) {
            let _ = std::fs::remove_file(&temp_path);
            eprintln!("CSV row write error: {error}");
            return Err(CommandError::new(
                "export_failed",
                "No se pudieron escribir las filas del archivo CSV.",
            ));
        }
    }

    if let Err(error) = writer.flush() {
        let _ = std::fs::remove_file(&temp_path);
        eprintln!("CSV flush error: {error}");
        return Err(CommandError::new(
            "export_failed",
            "No se pudo finalizar el archivo CSV.",
        ));
    }
    drop(writer);

    finalize_export_destination(&final_path, &temp_path)
}
