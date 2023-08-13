<script setup>
import { reactive, onMounted, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { tokenStore } from '../store.js'


const router = useRouter()
const route = useRoute()

const goodDetail = ref({});

const addOrderStatus = ref('');
const addOrderResult = ref({});

async function postData(url = '', data = {}) {
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': tokenStore.token.token.token_type + ' ' + tokenStore.token.token.access_token,
    },
    body: JSON.stringify(data)
  });
  return response.json();
}

async function fetch_goods_detail() {
  const data = await fetch(
    'http://127.0.0.1:3004/goods_detail?goods_id=' + route.params.id,
    {
      method: "post",
      headers: {
        'Content-Type': 'application/json'
      },
    }
  ).then(rsp => rsp.json())
  // alert.call("")
  // window.alert("get goods_list over.");

  goodDetail.value = data;
}


async function add_order() {
  addOrderStatus.value = 'check sign status....';
  if (JSON.stringify(tokenStore.token) === '{}') {
    window.alert("please sign in first.");
    return
  }
  addOrderStatus.value = 'request token....';

  const addOrderTokenResp = await postData(
    'http://127.0.0.1:3002/request_order_token',
    {}
  )
  // .then(rsp => rsp.json())

  const addOrderToken = addOrderTokenResp.token;
  addOrderStatus.value = 'request token success.' + addOrderToken;


  const addOrderResp = await postData(
    'http://127.0.0.1:3002/add_order',
    {
      'items_id': Number(route.params.id),
      'price': goodDetail.value.unit_price,
      'count': 1,
      'currency': "CNY",
      'description': '',
      'token': addOrderToken,
    }
  )
  // .then(rsp => rsp.json())

  addOrderStatus.value = 'add order success.';

  // alert.call("")
  // window.alert("get goods_list over.");

  addOrderResult.value = addOrderResp;
  // goodDetail.value = data;
}

onMounted((async () => {
  fetch_goods_detail()
}))

</script>

<template>
  <main>
    <div>
      Goods Detail:
    </div>
    <div>
      <p>name:</p>
      <p>{{ goodDetail.goods_name }}</p>
      <p>des:</p>
      <p>{{ goodDetail.goods_des }}</p>

      <div>
        <button @click="add_order">buy now!</button>
      </div>

      <br>
      <p>{{ addOrderStatus }}</p>
      <p>{{ JSON.stringify(addOrderResult) }}</p>
    </div>
  </main>
</template>
