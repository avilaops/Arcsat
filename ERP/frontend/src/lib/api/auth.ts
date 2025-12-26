import axios from "axios";
import { z } from "zod";

import api from "./client";

export const LoginPayloadSchema = z.object({
  email: z.string().email(),
  password: z.string().min(6),
  tenant_domain: z.string(),
  mfa_token: z.string().optional()
});

export type LoginPayload = z.infer<typeof LoginPayloadSchema>;

export const AuthUserSchema = z.object({
  id: z.string(),
  name: z.string(),
  email: z.string().email(),
  roles: z.array(z.string()),
  tenant_id: z.string(),
  avatar_url: z.string().nullable().optional()
});

export type AuthUser = z.infer<typeof AuthUserSchema>;

export const LoginResponseSchema = z.object({
  access_token: z.string(),
  refresh_token: z.string(),
  expires_in: z.number(),
  user: AuthUserSchema,
  requires_mfa: z.boolean().optional()
});

export type LoginResponse = z.infer<typeof LoginResponseSchema>;

export async function login(payload: LoginPayload) {
  const response = await api.post("/auth/login", payload);
  return LoginResponseSchema.parse(response.data);
}

export async function refresh(refreshToken: string) {
  const response = await axios.post(
    "/api/v1/auth/refresh",
    { refresh_token: refreshToken },
    { timeout: 15000 }
  );
  return LoginResponseSchema.parse(response.data);
}

export async function logout() {
  await api.post("/auth/logout");
}
