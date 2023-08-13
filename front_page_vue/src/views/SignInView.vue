<script setup>
import { reactive, onMounted, ref } from 'vue'
import { tokenStore } from '../store.js'


const userEmail = ref("");
const userPassword = ref("");

const signedToken = ref({});

async function sign_in(event) {
  const data = await fetch(
    `http://127.0.0.1:3003/sign_in`,
    {
      method: "post",
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        email: userEmail.value,
        password: userPassword.value
      })
    }
  ).then(rsp => rsp.json())
  // alert.call("")
  window.alert("sign_in over uuid:" + data.uid);

  for (var key in data) {
    console.log("sign_in get data: key" + key + " : " + data[key])
  }
  console.log("sign_in get data,access_token: " + data.token.access_token)
  console.log("sign_in get data,token_type " + data.token.token_type)

  signedToken.value = data;
  tokenStore.token = data;

  let jwtStr = JSON.stringify(data);
  console.log("setToken save to localStorage:" + jwtStr);
  localStorage.setItem("JwtKey", jwtStr);
}
</script>

<template>
  <main>
    <title>Sign In</title>

    <div>
      <p>userEmail:</p>
      <input type="text" v-model="userEmail" />
    </div>
    <div>
      <p>password:</p>
      <input type="password" v-model="userPassword" />
    </div>
    <div>
      <button @click="sign_in">sign in</button>
    </div>
    <p>
      {{ signedToken }}
    </p>
  </main>
</template>
