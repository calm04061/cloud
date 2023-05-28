<script setup lang="ts">
import {getAllMeta, addMeta, updateMeta,deleteMeta} from "@/api/cloudApi";
import {useRouter} from 'vue-router'
const desserts = ref([]);

const list = async () => {
  let a = await getAllMeta();
  desserts.value = a.data.data;
};

onMounted(() => {
  const { currentRoute } = useRouter();
  const route = currentRoute.value;
  let callback=route.query.callback
  if (callback){
    opener.location.reload();
    self.close()
  }
  console.log(list());
});
const cloud_types = ref([{id: 1, name: "阿里云盘"}, {id: 2, name: "百度"}, {id: 3, name: "本地磁盘"}, {id: 4, name: "OneDrive"}])
const status = ref([{id: 0, name: "待初始化"}, {id: 1, name: "待配置根目录"}, {id: 2, name: "可用"}, {
  id: 3,
  name: "token失效"
}, {id: 4, name: "禁用"}])
const dialog = ref(false);
const search = ref("");
const editedIndex = ref(-1);
const refForm = ref();
const editedItem = ref({
  id: "",
  name: "1.jpg",
  status: "",
  cloud_type: "",
  total_quota: "",
  used_quota: "",
  remaining_quota: "",
});
const defaultItem = ref({
  id: "",
  name: "",
  status: "",
  cloud_type: "",
  total_quota: "",
  used_quota: "",
  remaining_quota: "",
});

const nameRules = [
  (v) => !!v || "Name is required",
  (v) => (v && v.length <= 10) || "Name must be less than 10 characters",
];
// const convertVolume = computed((v) => {
//   return desserts.value.filter((user: any) => {
//     return user.name.toLowerCase().includes(search.value.toLowerCase());
//   });
// });
const unit = ["B", "KB", "MB", "GB", "TB", "EB"];

function convertVolume(v) {
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
function convertType(v) {
  for (let i = 0; i < cloud_types.value.length; i++) {
    let item = cloud_types.value[i];
    if (item.id == v) {
      return item.name;
    }
  }
}
function convertStatus(v) {
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
  // const index = desserts.value.indexOf(item);
  // confirm("Are you sure you want to delete this item?") &&
  // desserts.value.splice(index, 1);
  // ``;
  await list();
}

function authItem(item: any) {
  window.location.href="/api/authorize/storage/" + item.id
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
    await updateMeta(editedItem.value,editedItem.value.id);
    // Object.assign(desserts.value[editedIndex.value], editedItem.value);
  } else {
    await addMeta(editedItem.value);
    // desserts.value.push(editedItem.value);
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
                <v-btn color="primary" v-bind="props" flat class="ml-auto">
                  <v-icon class="mr-2">mdi-account-multiple-plus</v-icon>
                  Add
                  Cloud
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
                          v-model="editedItem.cloud_type"
                          label="类型"
                          item-value="id"
                          item-title="name"
                          :items="cloud_types"
                        ></v-select>
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
                    :disabled="
                      editedItem.username == '' || editedItem.usermail == ''
                    "
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
              <div>
                <v-img
                  :src="item.avatar"
                  width="40"
                  class="rounded-circle img-fluid"
                ></v-img>
              </div>

              <div class="ml-5">
                <p class="font-weight-bold">{{ item.name }}</p>
                <!--                  <span class="d-block mt-1 text-caption textSecondary">{{-->
                <!--                    item.usermail-->
                <!--                  }}</span>-->
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
