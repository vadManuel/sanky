export type RpcType =
  | "unary"
  | "server-streaming"
  | "client-streaming"
  | "bidirectional-streaming";

export type StreamingSignal = "cancel" | "end" | "pause" | "resume";

export interface ServiceMethod {
  name: string;
  type: RpcType;
  inputType: string;
  outputType: string;
}

export interface Service {
  name: string;
  methods: ServiceMethod[];
}

export interface RequestForm {
  address: string;
  requestData: string;
  streamingData?: string;
}

export interface GrpcResponse {
  success: boolean;
  response?: any;
  error?: string;
  metadata?: any;
}

export interface StreamingResponse {
  success: boolean;
  data?: any;
  error?: string;
  done?: boolean;
  metadata?: any;
  direction?: "sent" | "received";
}
