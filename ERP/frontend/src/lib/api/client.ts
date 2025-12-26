import axios from "axios";

const api = axios.create({
  baseURL: "/api/v1",
  timeout: 15000
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem("erp.accessToken");
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export type ApiClientConfig = {
  baseURL?: string;
};

export function configureApiClient(config: ApiClientConfig) {
  if (config.baseURL) {
    api.defaults.baseURL = config.baseURL;
  }
}

export default api;
