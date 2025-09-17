export interface JsonFormatResult {
  success: boolean;
  formatted: string;
  error?: string;
}

export function formatJson(
  jsonString: string,
  indent: number = 2
): JsonFormatResult {
  if (!jsonString.trim()) {
    return { success: true, formatted: jsonString };
  }
  try {
    const parsed = JSON.parse(jsonString);
    return { success: true, formatted: JSON.stringify(parsed, null, indent) };
  } catch (error) {
    return {
      success: false,
      formatted: jsonString,
      error: error instanceof Error ? error.message : "Invalid JSON format",
    };
  }
}

export function minifyJson(jsonString: string): JsonFormatResult {
  if (!jsonString.trim()) return { success: true, formatted: jsonString };
  try {
    const parsed = JSON.parse(jsonString);
    return { success: true, formatted: JSON.stringify(parsed) };
  } catch (error) {
    return {
      success: false,
      formatted: jsonString,
      error: error instanceof Error ? error.message : "Invalid JSON format",
    };
  }
}
