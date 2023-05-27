import axios from "axios";
// import { useSnackbarStore } from "@/stores/snackbarStore";
const apiInstance = axios.create({
  baseURL: "/api",
  timeout: 100000,
});

// apiInstance.interceptors.response.use(
//   (response) => {
//     return response;
//   },
//   (error) => {
//     const snackbarStore = useSnackbarStore();
//     if (error.response) {
//       // const status = error.response.status;
//       const data = error.response.data;
//       snackbarStore.showErrorMessage(data.error);
//     } else {
//       snackbarStore.showErrorMessage("Network Error");
//     }
//     return Promise.reject(error);
//   }
// );

// Get all models.
export const getAllMeta = () => {
  return apiInstance.get("/storage/meta", {
  });
};
// Get all models.
export const addMeta = (meta:any) => {
  return apiInstance.post("/storage/meta", {name:meta.name,cloud_type:meta.cloud_type}, {});
};// Get all models.
export const updateMeta = (meta:any,id:any) => {
  return apiInstance.post("/storage/meta/"+id, {name:meta.name,cloud_type:meta.cloud_type}, {});
};export const deleteMeta = (id:any) => {
  return apiInstance.delete("/storage/meta/"+id, {});
};

// Get account balance information.
export const getBalanceApi = (apiKey: string) => {
  return apiInstance.get("/dashboard/billing/credit_grants", {
    headers: {
      Authorization: "Bearer " + apiKey,
    },
  });
};

// speech-to-text
export const createTranscriptionApi = (formData: any, apiKey: string) => {
  return apiInstance.post("/v1/audio/transcriptions", formData, {
    headers: {
      Authorization: "Bearer " + apiKey,
    },
  });
};

// completions(Stream UnUsed)
export const createCompletionApi = (data: any, apiKey: string) => {
  return apiInstance.post("/v1/chat/completions", data, {
    headers: {
      Authorization: "Bearer " + apiKey,
    },
  });
};
