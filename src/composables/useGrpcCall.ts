import { ref, onMounted, onUnmounted } from "vue";
import type {
  GrpcResponse,
  RequestForm,
  ServiceMethod,
  StreamingResponse,
  RpcType,
  StreamingSignal,
} from "../types/grpc";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useGrpcCall() {
  const loading = ref(false);
  const lastResponse = ref<GrpcResponse | null>(null);
  const streamingResponses = ref<StreamingResponse[]>([]);
  const isStreaming = ref(false);

  const makeGrpcCall = async (
    selectedMethod: ServiceMethod & { serviceName?: string },
    formData: RequestForm,
    protoContent: string
  ) => {
    const rpcType = selectedMethod.type as RpcType;
    if (rpcType !== "unary")
      return makeStreamingCall(selectedMethod, formData, protoContent);

    loading.value = true;
    try {
      const requestData = JSON.parse(formData.requestData);
      const full_method = `${selectedMethod.serviceName}.${selectedMethod.name}`;
      const result = await invoke<unknown>("grpc_invoke_unary", {
        params: {
          address: formData.address,
          full_method,
          request_json: requestData,
          proto_content: protoContent || null,
          insecure: true,
        },
      });
      lastResponse.value = { success: true, response: result };
    } catch (e: any) {
      lastResponse.value = { success: false, error: String(e) };
    } finally {
      loading.value = false;
    }
  };

  const makeStreamingCall = async (
    selectedMethod: ServiceMethod & { serviceName?: string },
    formData: RequestForm,
    protoContent: string
  ) => {
    loading.value = true;
    isStreaming.value = true;
    streamingResponses.value = [];
    try {
      const requestData = formData.requestData
        ? JSON.parse(formData.requestData)
        : undefined;
      const streamingData = formData.streamingData
        ? JSON.parse(formData.streamingData)
        : undefined;
      await invoke("grpc_make_streaming_call", {
        params: {
          address: formData.address,
          method: `${selectedMethod.serviceName}.${selectedMethod.name}`,
          request_data: requestData,
          streaming_data: streamingData,
          proto_content: protoContent || null,
          rpc_type: selectedMethod.type,
        },
      });
    } catch (e) {
      isStreaming.value = false;
    } finally {
      loading.value = false;
    }
  };

  const sendStreamingSignal = async (
    selectedMethod: ServiceMethod & { serviceName?: string },
    formData: RequestForm,
    protoContent: string,
    signal: StreamingSignal
  ) => {
    await invoke("grpc_send_streaming_signal", {
      params: {
        address: formData.address,
        method: `${selectedMethod.serviceName}.${selectedMethod.name}`,
        signal,
      },
    });
    if (signal === "end" || signal === "cancel") isStreaming.value = false;
  };

  const sendStreamingMessage = async (
    selectedMethod: ServiceMethod & { serviceName?: string },
    formData: RequestForm,
    messageData: any
  ) => {
    streamingResponses.value.push({
      success: true,
      data: messageData,
      direction: "sent",
      done: false,
    });
    await invoke("grpc_send_streaming_message", {
      params: {
        address: formData.address,
        method: `${selectedMethod.serviceName}.${selectedMethod.name}`,
        message: messageData,
      },
    });
  };

  const handleStreamingData = (data: any) => {
    streamingResponses.value.push({
      success: true,
      data,
      done: false,
      direction: "received",
    });
  };
  const handleStreamingError = (error: any) => {
    streamingResponses.value.push({
      success: false,
      error: String(error),
      done: false,
    });
    isStreaming.value = false;
  };
  const handleStreamingEnd = () => {
    streamingResponses.value.push({ success: true, done: true });
    isStreaming.value = false;
  };

  let unsubs: Array<() => void> = [];
  onMounted(async () => {
    const u1 = await listen("streaming-data", (e) =>
      handleStreamingData(e.payload)
    );
    const u2 = await listen("streaming-error", (e) =>
      handleStreamingError(e.payload)
    );
    const u3 = await listen("streaming-end", () => handleStreamingEnd());
    unsubs = [() => u1(), () => u2(), () => u3()];
  });
  onUnmounted(() => {
    unsubs.forEach((u) => u());
    unsubs = [];
  });

  const clearResponse = () => {
    lastResponse.value = null;
    streamingResponses.value = [];
    isStreaming.value = false;
  };

  return {
    loading,
    lastResponse,
    streamingResponses,
    isStreaming,
    makeGrpcCall,
    makeStreamingCall,
    sendStreamingSignal,
    sendStreamingMessage,
    clearResponse,
    handleStreamingData,
    handleStreamingError,
    handleStreamingEnd,
  };
}
