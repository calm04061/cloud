import axios from "axios";
// import { useSnackbarStore } from "@/stores/snackbarStore";
const apiInstance = axios.create({
    baseURL: "/api",
    timeout: 100000,
});


// Get all models.
export const getSupportCloud = () => {
    return apiInstance.get("/support/cloud/types", {});
};
export const getAllMeta = () => {
    return apiInstance.get("/storage/meta", {});
};
// Get all models.
export const addMeta = (meta: any) => {
    return apiInstance.post("/storage/meta", {
        name: meta.name,
        cloud_type: parseInt(meta.cloud_type),
        data_root: meta.data_root,
        auth: JSON.stringify(meta.auth)
    }, {});
};// Get all models.
export const updateMeta = (meta: any, id: any) => {
    return apiInstance.post("/storage/meta/" + id, {
        name: meta.name, cloud_type: parseInt(meta.cloud_type),
        data_root: meta.data_root,
        auth: JSON.stringify(meta.auth)
    }, {});
};
export const deleteMeta = (id: any) => {
    return apiInstance.delete("/storage/meta/" + id, {});
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
