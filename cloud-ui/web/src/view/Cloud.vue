<script setup lang="ts">
import {ref, onMounted, computed} from 'vue'
import {addMeta, deleteMeta, getAllMeta, getSupportCloud, updateMeta} from "../api/cloudApi";
import {useRouter} from 'vue-router'

interface CloudMeta {
  id: number,
  name: string,
  status: number,
  auth: any,
  cloud_type: number,
  total_quota: number,
  used_quota: number,
  data_root: string,
  remaining_quota: number,
}

const desserts = ref<CloudMeta[]>([]);
const list = async () => {
  let meta = await getAllMeta();
  let temp = meta.data.data;
  temp = temp.map((val:any) => {
    val.auth = JSON.parse(val.auth)
    val.cloud_type = val.cloud_type.toString()
    return val;
  });
  desserts.value = temp;
};

onMounted(async () => {
  const {currentRoute} = useRouter();
  const route = currentRoute.value;
  let callback = route.query.callback
  if (callback) {
    opener.location.reload();
    self.close()
  }
  let temp = await getSupportCloud();
  cloud_types.value = temp.data.data;
  await list();
});
const cloud_types = ref()
const status = ref([{id: 0, name: "待初始化"}, {id: 1, name: "待配置根目录"}, {id: 2, name: "可用"}, {
  id: 3,
  name: "token失效"
}, {id: 4, name: "禁用"}])
const dialog = ref(false);
const search = ref("");
const editedIndex = ref(-1);
const refForm = ref();
const editedItem = ref<CloudMeta>({
  id: 0,
  name: "",
  status: 0,
  auth: {},
  cloud_type: 1,
  total_quota: 0,
  used_quota: 0,
  data_root: "",
  remaining_quota: 0,
});
const defaultItem = ref<CloudMeta>({
  id: 0,
  name: "",
  status: 0,
  auth: {},
  cloud_type: 1,
  total_quota: 0,
  data_root: "",
  used_quota: 0,
  remaining_quota: 0,
});

// const nameRules = [
//   (v) => !!v || "Name is required",
//   (v) => (v && v.length <= 10) || "Name must be less than 10 characters",
// ];
// const convertVolume = computed((v) => {
//   return desserts.value.filter((user: any) => {
//     return user.name.toLowerCase().includes(search.value.toLowerCase());
//   });
// });
const unit = ["B", "KB", "MB", "GB", "TB", "EB"];

function convertVolume(v: number) {
  if (!v) {
    return v;
  }
  let i = 0;
  while (v > 0) {
    let temp = v / 1024;
    if (temp <= 2) {
      return v + unit[i];
    }
    i++;
    v = parseInt(temp.toString());
  }
  return v;
}

function convertType(v: number) {
  for (let i = 0; i < cloud_types.value.length; i++) {
    let item = cloud_types.value[i];
    if (item.id == v) {
      return item.name;
    }
  }
}

function convertStatus(v: number) {
  for (let i = 0; i < status.value.length; i++) {
    let item = status.value[i];
    if (item.id == v) {
      return item.name;
    }
  }
}

//Methods
const filteredList = computed(() => {
  return desserts.value.filter((user: any) => {
    return user.name.toLowerCase().includes(search.value.toLowerCase());
  });
});

function editItem(item: any) {
  editedIndex.value = desserts.value.indexOf(item);
  editedItem.value = Object.assign({}, item);
  dialog.value = true;
}

async function deleteItem(item: any) {
  await deleteMeta(item.id);
  await list();
}

function authItem(item: any) {
  window.location.href = "/api/authorize/storage/" + item.id
}

function close() {
  dialog.value = false;
  setTimeout(() => {
    editedItem.value = Object.assign({}, defaultItem.value);
    editedIndex.value = -1;
  }, 300);
}

async function save() {
  if (editedIndex.value > -1) {
    await updateMeta(editedItem.value, editedItem.value.id);
  } else {
    await addMeta(editedItem.value);
  }
  await list();
  close();
}

//Computed Property
const formTitle = computed(() => {
  return editedIndex.value === -1 ? "New Cloud" : "Edit Cloud";
});
</script>
<template>
  <v-container>
    <v-card>
      <v-card-text>
        <v-row>
          <v-col cols="12" lg="4" md="6">
            <v-text-field
                density="compact"
                v-model="search"
                label="Search Cloud"
                hide-details
                variant="outlined"
                color="primary"
            ></v-text-field>
          </v-col>
          <v-col cols="12" lg="8" md="6" class="text-right">
            <v-dialog v-model="dialog" max-width="700">
              <template v-slot:activator="{ props }">
                <v-btn color="primary" v-bind="props" :flat="true" class="ml-auto" @click="editItem(defaultItem)">
                  <v-icon class="mr-2">mdi-account-multiple-plus</v-icon>
                  Add Cloud
                </v-btn>
              </template>
              <v-card>
                <v-card-title class="pa-4 bg-secondary">
                  <span class="title text-white">{{ formTitle }}</span>
                </v-card-title>
                <v-card-text>
                  <v-form
                      class="mt-5"
                      ref="form"
                      v-model="refForm"
                      lazy-validation
                  >
                    <v-row>
                      <v-col cols="12" sm="6">
                        <v-text-field
                            variant="outlined"
                            color="primary"
                            required
                            v-model="editedItem.name"
                            label="名称"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6">
                        <v-select
                            variant="outlined"
                            color="primary"
                            :readonly="editedItem.id>0"
                            v-model="editedItem.cloud_type"
                            label="类型"
                            item-value="id"
                            item-title="name"
                            :items="cloud_types"
                        ></v-select>
                      </v-col>
                    </v-row>
                    <v-row>
                      <v-col cols="12" sm="6" :hidden="editedItem.cloud_type!=5">
                        <v-text-field
                            variant="outlined"
                            color="primary"
                            required
                            v-model="editedItem.auth.username"
                            label="用户名"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" :hidden="editedItem.cloud_type!=5">
                        <v-text-field
                            variant="outlined"
                            color="primary"
                            type="password"
                            required
                            v-model="editedItem.auth.password"
                            label="密码"
                        ></v-text-field>
                      </v-col>
                    </v-row>
                    <v-row>
                      <v-col cols="12" sm="6" :hidden="editedItem.cloud_type!=5">
                        <v-text-field
                            variant="outlined"
                            color="primary"
                            required
                            v-model="editedItem.auth.hostname"
                            label="主机"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" :hidden="editedItem.cloud_type!=5">
                        <v-text-field
                            variant="outlined"
                            color="primary"
                            required
                            v-model="editedItem.auth.port"
                            label="端口"
                        ></v-text-field>
                      </v-col>
                    </v-row>
                    <v-row>
                      <v-col cols="12" :hidden="!(editedItem.cloud_type==3||editedItem.cloud_type==5)">
                        <v-text-field
                            variant="outlined"
                            color="primary"
                            required
                            v-model="editedItem.data_root"
                            label="存储路径"
                        ></v-text-field>
                      </v-col>
                    </v-row>

                  </v-form>
                </v-card-text>
                <v-divider></v-divider>
                <v-card-actions class="pa-4">
                  <v-spacer></v-spacer>
                  <v-btn color="error" @click="close">Cancel</v-btn>
                  <v-btn
                      color="secondary"
                      variant="flat"
                      @click="save"
                  >Save
                  </v-btn
                  >
                </v-card-actions>
              </v-card>
            </v-dialog>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>

    <v-card class="mt-2">
      <v-table class="mt-5">
        <thead>
        <tr>
          <th class="text-subtitle-1 font-weight-semibold">Id</th>
          <th class="text-subtitle-1 font-weight-semibold">名称</th>
          <th class="text-subtitle-1 font-weight-semibold">类型</th>
          <th class="text-subtitle-1 font-weight-semibold">状态</th>
          <th class="text-subtitle-1 font-weight-semibold">根目录</th>
          <th class="text-subtitle-1 font-weight-semibold">总容量</th>
          <th class="text-subtitle-1 font-weight-semibold">已使用容量</th>
          <th class="text-subtitle-1 font-weight-semibold">剩余容量</th>
          <th class="text-subtitle-1 font-weight-semibold"></th>
        </tr>
        </thead>
        <tbody class="text-body-1">
        <tr v-for="item in filteredList" :key="item.id">
          <td class="font-weight-bold">{{ item.id }}</td>
          <td>
            <div class="d-flex align-center py-1">
              <div class="ml-5">
                <p class="font-weight-bold">{{ item.name }}</p>
              </div>
            </div>
          </td>
          <td v-text="convertType(item.cloud_type)"/>
          <td v-text="convertStatus(item.status)"/>
          <td>{{ item.data_root }}</td>
          <td v-text="convertVolume(item.total_quota)"/>
          <td v-text="convertVolume(item.used_quota)"/>
          <td v-text="convertVolume(item.remaining_quota )"/>
          <td>
            <div class="d-flex align-center">
              <v-tooltip text="Edit">
                <template v-slot:activator="{ props }">
                  <v-btn
                      color="blue"
                      icon
                      variant="text"
                      @click="editItem(item)"
                      v-bind="props"
                  >
                    <v-icon>mdi-pencil-outline</v-icon>
                  </v-btn>
                </template>
              </v-tooltip>
              <v-tooltip text="Delete">
                <template v-slot:activator="{ props }">
                  <v-btn
                      icon
                      variant="text"
                      @click="deleteItem(item)"
                      v-bind="props"
                      color="error"
                  >
                    <v-icon>mdi-delete-outline</v-icon>
                  </v-btn>
                </template>
              </v-tooltip>
              <v-tooltip text="授权">
                <template v-slot:activator="{ props }">
                  <v-btn
                      icon
                      variant="text"
                      @click="authItem(item)"
                      v-bind="props"
                      color="error"
                  >
                    <v-icon
                        size="large"
                        color="green-darken-2"
                        icon="mdi-domain"
                    ></v-icon>
                  </v-btn>
                </template>
              </v-tooltip>
            </div>
          </td>
        </tr>
        </tbody>
      </v-table>
    </v-card>
  </v-container>
</template>
