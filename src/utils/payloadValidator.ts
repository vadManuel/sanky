export interface ValidationError {
  field: string;
  message: string;
  type: "missing" | "type_mismatch" | "invalid_value" | "unknown_field";
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
}

export interface ProtoField {
  name: string;
  type: string;
  isRepeated: boolean;
  isRequired: boolean;
}

export interface ProtoMessage {
  name: string;
  fields: ProtoField[];
}

export function parseProtoMessage(
  messageName: string,
  protoContent: string
): ProtoMessage | null {
  try {
    const messageRegex = new RegExp(
      `message\\s+${messageName}\\s*{([^}]*(?:{[^}]*}[^}]*)*)}`,
      "s"
    );
    const match = protoContent.match(messageRegex);
    if (!match) return null;
    const messageBody = match[1];
    const fields: ProtoField[] = [];
    const fieldRegex = /(?:repeated\s+)?(\w+)\s+(\w+)\s*=\s*(\d+);/g;
    let fieldMatch;
    while ((fieldMatch = fieldRegex.exec(messageBody)) !== null) {
      const [, fieldType, fieldName] = fieldMatch;
      const isRepeated = fieldMatch[0].includes("repeated");
      const isRequired = false; // proto3 default
      fields.push({ name: fieldName, type: fieldType, isRepeated, isRequired });
    }
    return { name: messageName, fields };
  } catch {
    return null;
  }
}

export function validatePayload(
  payload: any,
  messageName: string,
  protoContent: string
): ValidationResult {
  const errors: ValidationError[] = [];
  const messageDef = parseProtoMessage(messageName, protoContent);
  if (!messageDef) {
    return {
      isValid: false,
      errors: [
        {
          field: "root",
          message: `Could not find message definition for '${messageName}'`,
          type: "invalid_value",
        },
      ],
    };
  }

  for (const field of messageDef.fields) {
    const fieldValue = payload[field.name];
    if (field.isRequired && (fieldValue === undefined || fieldValue === null)) {
      errors.push({
        field: field.name,
        message: `Required field '${field.name}' is missing`,
        type: "missing",
      });
      continue;
    }
    if (fieldValue === undefined || fieldValue === null) continue;

    const fieldError = validateFieldType(fieldValue, field, protoContent);
    if (fieldError) errors.push(fieldError);
  }

  const known = new Set(messageDef.fields.map((f) => f.name));
  for (const key in payload) {
    if (!known.has(key)) {
      errors.push({
        field: key,
        message: `Unknown field '${key}'`,
        type: "unknown_field",
      });
    }
  }

  return { isValid: errors.length === 0, errors };
}

function validateFieldType(
  value: any,
  field: ProtoField,
  protoContent: string
): ValidationError | null {
  const { name, type, isRepeated } = field;
  if (isRepeated) {
    if (!Array.isArray(value)) {
      return {
        field: name,
        message: `Field '${name}' should be an array (repeated field)`,
        type: "type_mismatch",
      };
    }
    for (let i = 0; i < value.length; i++) {
      const elementError = validateSingleValue(value[i], type, protoContent);
      if (elementError) {
        return {
          field: `${name}[${i}]`,
          message: elementError,
          type: "type_mismatch",
        };
      }
    }
    return null;
  }
  const singleValueError = validateSingleValue(value, type, protoContent);
  if (singleValueError)
    return { field: name, message: singleValueError, type: "type_mismatch" };
  return null;
}

function validateSingleValue(
  value: any,
  type: string,
  protoContent: string
): string | null {
  switch (type.toLowerCase()) {
    case "string":
      if (typeof value !== "string")
        return `Expected string, got ${typeof value}`;
      break;
    case "bool":
      if (typeof value !== "boolean")
        return `Expected boolean, got ${typeof value}`;
      break;
    case "double":
    case "float":
      if (typeof value !== "number")
        return `Expected number, got ${typeof value}`;
      break;
    case "int32":
    case "int64":
    case "uint32":
    case "uint64":
    case "sint32":
    case "sint64":
    case "fixed32":
    case "fixed64":
    case "sfixed32":
    case "sfixed64":
      if (typeof value !== "number" || !Number.isInteger(value))
        return `Expected integer, got ${typeof value}`;
      break;
    case "bytes":
      if (typeof value !== "string")
        return `Expected string (base64), got ${typeof value}`;
      break;
    default:
      if (isCustomMessageType(type, protoContent)) {
        if (typeof value !== "object" || value === null)
          return `Expected object (message type), got ${typeof value}`;
        const nested = validatePayload(value, type, protoContent);
        if (!nested.isValid)
          return `Invalid nested message: ${nested.errors[0]?.message}`;
      } else {
        return `Unknown type '${type}'`;
      }
  }
  return null;
}

function isCustomMessageType(typeName: string, protoContent: string): boolean {
  const messageRegex = new RegExp(`message\\s+${typeName}\\s*{`, "g");
  return messageRegex.test(protoContent);
}

export function formatValidationErrors(errors: ValidationError[]): string {
  if (errors.length === 0) return "";
  const msgs = errors.map((e) => {
    switch (e.type) {
      case "missing":
        return `❌ ${e.message}`;
      case "type_mismatch":
        return `⚠️ ${e.message}`;
      case "unknown_field":
        return `ℹ️ ${e.message}`;
      case "invalid_value":
        return `🚫 ${e.message}`;
      default:
        return `❓ ${e.message}`;
    }
  });
  return msgs.join("\n");
}
