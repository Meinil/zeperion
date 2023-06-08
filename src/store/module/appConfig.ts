import { defineStore } from 'pinia'
import { Modules } from '../module'

export default defineStore(Modules.APP_CONFIG, {
  state: () => {
    return { 
      javaHome: '123',
      themeColor: '#5a9cf8',
    }
  },
  actions: {
    
  },
})