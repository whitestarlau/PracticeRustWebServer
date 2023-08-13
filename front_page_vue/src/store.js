import { reactive } from 'vue'

/**
 * like this:
 * { "uid": "1b017638-1b1c-4e75-8a16-389f72dfa98e", "token": { "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxYjAxNzYzOC0xYjFjLTRlNzUtOGExNi0zODlmNzJkZmE5OGUiLCJleHAiOjE2OTI1MjQ0MTcsImlhdCI6MTY5MjQzODAxN30.QZcxjqcRADI5eRuKTgE4LhPrk0nbMuy9G66Iz8KcK0A", "token_type": "Bearer" } }
 */
export const tokenStore = reactive({
    token: {},
    setToken(jwt) {
        console.log("setToken");
        this.token = jwt;
    }
});

