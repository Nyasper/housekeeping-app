import { save as saveFileDialog } from '@tauri-apps/plugin-dialog';

import { errorMessage } from '#lib/api/errors.js';
import { call, isTauriRuntime } from '#lib/api/invoke.js';
import type { ExportFilters, ExportFormat, ExportKind } from '#lib/api/types.js';
import { todayIso } from '#lib/utils/format.js';

const FORMAT_LABELS: Record<ExportFormat, { name: string; extensions: string[]; title: string }> = {
  csv: {
    name: 'Archivo CSV (UTF-8)',
    extensions: ['csv'],
    title: 'Exportar datos a CSV'
  },
  xlsx: {
    name: 'Planilla Excel (*.xlsx)',
    extensions: ['xlsx'],
    title: 'Exportar planilla imprimible XLSX'
  },
  pdf: {
    name: 'Documento PDF (*.pdf)',
    extensions: ['pdf'],
    title: 'Exportar planilla imprimible PDF'
  }
};

export class ExportService {
  exporting = $state(false);
  activeFormat = $state<ExportFormat | null>(null);
  error = $state<string | null>(null);
  lastExportedPath = $state<string | null>(null);

  clearError(): void {
    this.error = null;
  }

  suggestedFileName(kind: ExportKind, filters: ExportFilters, format: ExportFormat): string {
    const fallbackDate = todayIso();
    switch (kind) {
      case 'daily_sheet':
        return `planilla-diaria-${filters.date || fallbackDate}.${format}`;
      case 'services_by_staff':
        return `servicios-por-persona-${filters.from || fallbackDate}-a-${filters.to || fallbackDate}.${format}`;
      case 'average_times':
        return `tiempos-promedio-${filters.from || fallbackDate}-a-${filters.to || fallbackDate}.${format}`;
      case 'absences_by_month':
        return `ausencias-${filters.month || fallbackDate.slice(0, 7)}.${format}`;
    }
  }

  async exportReport(
    kind: ExportKind,
    format: ExportFormat,
    filters: ExportFilters
  ): Promise<string | null> {
    if (!isTauriRuntime()) {
      this.error = 'La exportación de archivos solo está disponible en la aplicación de escritorio.';
      return null;
    }

    this.error = null;
    const meta = FORMAT_LABELS[format];
    const defaultPath = this.suggestedFileName(kind, filters, format);

    let selectedPath: string | null = null;
    try {
      selectedPath = await saveFileDialog({
        title: meta.title,
        defaultPath,
        filters: [
          {
            name: meta.name,
            extensions: meta.extensions
          }
        ]
      });
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    }

    if (!selectedPath) {
      return null;
    }

    const command =
      format === 'csv' ? 'export_csv' : format === 'xlsx' ? 'export_xlsx' : 'export_pdf';

    this.exporting = true;
    this.activeFormat = format;
    try {
      const saved = await call<string>(command, {
        kind,
        filters,
        destPath: selectedPath
      });
      this.lastExportedPath = saved;
      return saved;
    } catch (cause) {
      this.error = errorMessage(cause);
      return null;
    } finally {
      this.exporting = false;
      this.activeFormat = null;
    }
  }
}

export const exportService = new ExportService();
