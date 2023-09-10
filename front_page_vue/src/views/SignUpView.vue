<script setup>
import { reactive, onMounted, ref } from 'vue'

const userEmail = ref("");
const userPassword = ref("");
const userPassword2 = ref("");

async function sign_up(event) {
  if(userPassword.value != userPassword2.value) {
    window.alert("Two password is different,please check.");
    return
  }
  const data = await fetch(
    `http://127.0.0.1:3003/sign_up`,
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
  window.alert("sign_up over uuid:" + data.uid);

  console.log("sign_up get data: " + data)
}
</script>

<template>
  <main>
    <title>Sign Up</title>
    <div>
      <p>userEmail:</p>
      <input type="text" v-model="userEmail" />
    </div>
    <div>
      <p>password:</p>
      <input type="password" v-model="userPassword" />
    </div>
    <div>
      <p>check password again:</p>
      <input type="password" v-model="userPassword2" />
    </div>
    <div>
      <button @click="sign_up">sign up</button>
    </div>
  </main>
</template>
