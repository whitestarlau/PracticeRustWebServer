<script setup>
import HelloWorld from './components/HelloWorld.vue'
import TheWelcome from './components/TheWelcome.vue'
import { RouterLink, RouterView } from 'vue-router'

import { reactive, onMounted, ref } from 'vue'

// const hits = ref([]);

// onMounted((async () => {
//   const data = await fetch(
//     `https://hn.algolia.com/api/v1/search?query=vue`
//   ).then(rsp => rsp.json())
//   console.log("liuyx on get data: " + data.hits)
//   hits.value = data.hits
// }))

</script>

<template>
  <header>
    <img alt="Vue logo" class="logo" src="./assets/logo.svg" width="125" height="125" />

    <div class="wrapper">
      <HelloWorld msg="Rust mall demo." />
      <nav>
        <RouterLink to="/sign_in">signIn</RouterLink>
        <RouterLink to="/sign_up">signUp</RouterLink>
      </nav>
    </div>
  </header>

  <main>
    <!-- <TheWelcome /> -->

    <!-- <ul>
      <li v-for="item of hits" :key="item.objectID">
        <a :href="item.url">{{ item.title }}</a>
      </li>
    </ul> -->

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
    place-items: center;
    padding-right: calc(var(--section-gap) / 2);
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
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
