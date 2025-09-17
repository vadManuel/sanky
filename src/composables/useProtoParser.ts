import { computed, ref } from "vue";
import type { Service, ServiceMethod } from "../types/grpc";

export function useProtoParser() {
  const protoContent = ref("");

  const packageName = computed(() => {
    if (!protoContent.value) return "";
    const match = protoContent.value.match(/\bpackage\s+([A-Za-z0-9_\.]+)\s*;/);
    return match ? match[1] : "";
  });

  const parsedServices = computed(() => {
    if (!protoContent.value) return [] as Service[];

    const services: Service[] = [];
    const lines = protoContent.value.split("\n");

    let currentService: Service | null = null;

    for (const line of lines) {
      const trimmed = line.trim();

      if (trimmed.startsWith("service ")) {
        const match = trimmed.match(/service\s+(\w+)/);
        if (match) {
          const svcName = match[1];
          const fullName = packageName.value
            ? `${packageName.value}.${svcName}`
            : svcName;
          currentService = { name: fullName, methods: [] };
          services.push(currentService);
        }
      }

      if (trimmed.includes("rpc ") && currentService) {
        const match = trimmed.match(
          /rpc\s+(\w+)\s*\(\s*(?:stream\s+)?(\w+)\s*\)\s*returns\s*\(\s*(?:stream\s+)?(\w+)\s*\)/
        );
        if (match) {
          const [, methodName, inputType, outputType] = match;
          let methodType: ServiceMethod["type"] = "unary";
          if (trimmed.includes("stream")) {
            const inputStream = /rpc\s+\w+\s*\(\s*stream\s+\w+/.test(trimmed);
            const outputStream = /returns\s*\(\s*stream\s+\w+/.test(trimmed);
            if (inputStream && outputStream)
              methodType = "bidirectional-streaming";
            else if (outputStream) methodType = "server-streaming";
            else if (inputStream) methodType = "client-streaming";
          }
          currentService.methods.push({
            name: methodName,
            type: methodType,
            inputType,
            outputType,
          });
        }
      }
    }

    return services;
  });

  const generateSampleDataFromProto = (
    messageType: string,
    content: string
  ): Record<string, unknown> => {
    if (!messageType || !content) return { sample_field: "sample_value" };
    try {
      const messageRegex = new RegExp(
        `message\\s+${messageType}\\s*{([^}]+)}`,
        "s"
      );
      const match = content.match(messageRegex);
      if (!match) return { sample_field: "sample_value" };
      const body = match[1];
      const result: Record<string, unknown> = {};
      const fieldRegex = /(?:repeated\s+)?(\w+)\s+(\w+)\s*=\s*(\d+);/g;
      let m: RegExpExecArray | null;
      while ((m = fieldRegex.exec(body)) !== null) {
        const [, fieldType, fieldName] = m;
        const isRepeated = m[0].includes("repeated");
        const value = generateSampleValueForType(
          fieldType,
          content,
          isRepeated
        );
        result[fieldName] = value;
      }
      return result;
    } catch {
      return { sample_field: "sample_value" };
    }
  };

  const generateSampleValueForType = (
    fieldType: string,
    content: string,
    isRepeated = false
  ): unknown => {
    if (isRepeated) return [getDefaultValueForType(fieldType, content)];
    return getDefaultValueForType(fieldType, content);
  };

  const getDefaultValueForType = (
    fieldType: string,
    content: string
  ): unknown => {
    switch (fieldType.toLowerCase()) {
      case "string":
        return "sample";
      case "bool":
        return false;
      case "double":
      case "float":
        return 1.0;
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
        return 1;
      case "bytes":
        return "c2FtcGxl";
      default:
        if (isCustomMessageType(fieldType, content)) {
          return generateSampleDataFromProto(fieldType, content);
        }
        return "sample";
    }
  };

  const isCustomMessageType = (typeName: string, content: string) => {
    const messageRegex = new RegExp(`message\\s+${typeName}\\s*{`, "g");
    return messageRegex.test(content);
  };

  const generateSampleDataForMethod = (
    method: ServiceMethod,
    content: string
  ): unknown => generateSampleDataFromProto(method.inputType, content);

  const getMethodType = (type: string) => {
    const map: Record<string, string> = {
      unary: "primary",
      "server-streaming": "warning",
      "client-streaming": "info",
      "bidirectional-streaming": "success",
    };
    return map[type] || "primary";
  };

  return {
    protoContent,
    parsedServices,
    generateSampleDataFromProto,
    generateSampleDataForMethod,
    getMethodType,
  };
}
