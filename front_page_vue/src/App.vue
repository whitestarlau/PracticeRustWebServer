<script setup>
import HelloWorld from './components/HelloWorld.vue'
import TheWelcome from './components/TheWelcome.vue'
import { RouterLink, RouterView } from 'vue-router'

import { reactive, onMounted, ref } from 'vue'
import { tokenStore } from './store.js'


function sign_out(event) {
  tokenStore.token = {};
  localStorage.setItem("JwtKey","");
}

onMounted(() => {
  console.log("onMounted.");

  const initTokenStr = localStorage.getItem("JwtKey");

  console.log("init token str:" + initTokenStr);

  if (initTokenStr !== "{}" && initTokenStr !== null) {
    let token = JSON.parse(initTokenStr);
    tokenStore.token = token;
  }
})

</script>

<template>
  <div>
    <header>
      <img alt="Vue logo" class="logo" src="./assets/logo.svg" width="125" height="125" />

      <h1 class="green">Rust mall {{ emptyToken }} {{ tokenStore.token.token_type }}</h1>

      <div>
        <nav>
          <div v-if="JSON.stringify(tokenStore.token) === '{}'">
            <RouterLink to="/sign_in">signIn</RouterLink>
            <RouterLink to="/sign_up">signUp</RouterLink>
          </div>
          <div v-else>
            <RouterLink to="/">goodsList</RouterLink>
            <button v-on:click="sign_out">sign out</button>
          </div>
        </nav>
      </div>

    </header>
  </div>

  <main>
    <RouterView />
  </main>
</template>

<style scoped>
header {
  line-height: 1.5;
}

.logo {
  display: block;
  margin: 0 auto 2rem;
}

@media (min-width: 1024px) {
  header {
    display: flex;
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    justify-content: space-between;
    align-items: center;
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    flex-direction: row;
    /* place-items: flex-direction; */
    flex-wrap: wrap;
  }

  nav {
    width: 100%;
    font-size: 12px;
    text-align: center;
    margin-top: 2rem;
  }

  nav a.router-link-exact-active {
    color: var(--color-text);
  }

  nav a.router-link-exact-active:hover {
    background-color: transparent;
  }

  nav a {
    display: inline-block;
    padding: 0 1rem;
    border-left: 1px solid var(--color-border);
  }

  nav a:first-of-type {
    border: 0;
  }
}
</style>
