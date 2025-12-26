import { z } from "zod";
import api from "./client";

export const LeadStageEnum = z.enum([
  "new",
  "contacted",
  "qualification",
  "proposal",
  "negotiation",
  "won",
  "lost"
]);

export const LeadResponseSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  company: z.string().nullable(),
  email: z.string(),
  phone: z.string().nullable(),
  source: z.string(),
  stage: LeadStageEnum,
  score: z.number(),
  value: z.string(),
  probability: z.number(),
  owner: z.object({
    id: z.string().uuid(),
    name: z.string(),
    email: z.string().email()
  }),
  created_at: z.string().datetime(),
  last_contact: z.string().datetime().nullable()
});

export const PaginationSchema = z.object({
  current_page: z.number(),
  total_pages: z.number(),
  total_items: z.number(),
  per_page: z.number()
});

export const PaginatedLeadsSchema = z.object({
  data: z.array(LeadResponseSchema),
  pagination: PaginationSchema
});

export type LeadResponse = z.infer<typeof LeadResponseSchema>;
export type PaginatedLeads = z.infer<typeof PaginatedLeadsSchema>;

export async function fetchLeads(params: Record<string, unknown> = {}) {
  const response = await api.get("/crm/leads", { params });
  return PaginatedLeadsSchema.parse(response.data);
}
