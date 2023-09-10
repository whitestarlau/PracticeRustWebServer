<script setup>
import { reactive, onMounted, ref } from 'vue'

const page = ref(0);
const page_size = ref(10);

const goods = ref([]);

async function fetch_goods_list() {
  const data = await fetch(
    'http://127.0.0.1:3004/goods_list?page_size=' + page_size.value + '&page=' + page.value,
    {
      method: "post",
      headers: {
        'Content-Type': 'application/json'
      },
    }
  ).then(rsp => rsp.json())
  // alert.call("")
  // window.alert("get goods_list over.");

  goods.value = data;
}

onMounted((async () => {
  fetch_goods_list()
}))

</script>

<template>
  <main>
    <div>
      Goods List:
    </div>
    <div>
      <ul>
        <li v-for="item of goods" :key="item.objectID">
          <a :href="'/goods_detail/' + item.id">{{ item.goods_name }}</a>
        </li>
      </ul>
    </div>
  </main>
</template>
