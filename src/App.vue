<script setup lang="ts">
import { invoke, Channel } from "@tauri-apps/api/core"
import { Ref, ref, onMounted, onBeforeUnmount } from "vue"

type CefResult =
  | {
      tag: "path"
      data: {
        name: string
        path: string
        icon: string
        size: string
      }
    }
  | {
      tag: "count"
      data: number
    }
  | {
      tag: "totalSize"
      data: string
    }
  | {
      tag: "sign"
    }

const handleContextMenu = (event: MouseEvent) => {
  event.preventDefault()
}

onMounted(() => {
  document.addEventListener("contextmenu", handleContextMenu)
})

onBeforeUnmount(() => {
  document.removeEventListener("contextmenu", handleContextMenu)
})

const CefChannel = new Channel<CefResult>()
let count = ref()
let total_size = ref()
let paths: Ref<{ name: string; path: string; icon: string; size: string }[]> =
  ref([])
CefChannel.onmessage = (message) => {
  switch (message.tag) {
    case "path":
      paths.value.push(message.data)
      console.log(message.data)
      break
    case "count":
      count.value = message.data
      break
    case "totalSize":
      total_size.value = message.data
      break
    case "sign":
      paths.value.length = 0
      break
  }
}

let error: Ref<string[]> = ref([])

invoke("start", { channel: CefChannel }).catch((e) => {
  error.value.push(e as string)
  console.error(e)
})

function open_folder(path: string) {
  invoke("open_folder", {
    path,
  }).catch((e) => {
    error.value.push(e as string)
    console.error(e)
  })
}
</script>

<template>
  <div id="main" v-if="error.length == 0">
    <audio
      controls
      autoplay
      loop
      src="bgm.mp3"
      controlsList="nodownload"
    ></audio>
    <h2 class="title" id="title">
      这台电脑上总共有 {{ count }} 个使用了CEF的应用, CEF部分共占用
      {{ total_size }}
    </h2>
    <div id="paths_outer">
      <div id="paths_inner">
        <div id="path" v-for="path in paths" @click="open_folder(path.path)">
          <img
            class="path_icon"
            :src="'data:image/png;base64,' + path.icon"
            @dragstart.prevent
          />
          <p class="path_name">{{ path.name }}</p>
          <p class="path_size">{{ path.size }}</p>
        </div>
      </div>
    </div>
    <div id="info">
      <a id="repo" href="https://github.com/u3l6/CEF-Detector-Tauri"
        >Repo(求Star!)</a
      >
      <a id="website" href="https://u3l6.github.io/_">我的网站</a>
    </div>
  </div>
  <div class="title" v-if="error.length != 0">
    <h1>Error!</h1>
    <h1>{{ error }}</h1>
  </div>
</template>

<style scoped>
audio {
  height: 28px;
  width: 280px;
  bottom: 10px;
  right: 10px;
  border-radius: 100px;
  position: fixed;
  z-index: 100;
}
.title {
  user-select: text;
  position: fixed;
  left: 50%;
  transform: translateX(-50%);
  top: 17%;
  font-size: 30px;
  text-shadow:
    1px 1px 0 rgba(255, 255, 255, 0.6),
    -1px 1px 0 rgba(255, 255, 255, 0.6),
    1px -1px 0 rgba(255, 255, 255, 0.6),
    -1px -1px 0 rgba(255, 255, 255, 0.6);
}
#title {
  width: max-content;
}
#paths_outer {
  position: fixed;
  top: 27%;
  left: 100px;
  right: 100px;
  height: 65%;
}
#paths_inner {
  margin: auto;
  display: grid;
  grid-template-columns: repeat(auto-fit, 150px);
  grid-template-rows: repeat(auto-fit, 100px);
  gap: 10px;
  height: 100%;
  grid-auto-rows: 100px;
  overflow: auto;
  justify-content: center;
}
#path {
  backdrop-filter: blur(10px);
  border-radius: 10px;
  border-style: solid;
  border-width: 1px;
  border-color: rgba(255, 255, 255, 0.3);
}
#path:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  transition: box-shadow 0.3s;
}
#path:hover .path_icon {
  transform: scale(1.1);
  transition: 0.3s;
}
.path_icon {
  width: 50px;
  height: 50px;
  position: fixed;
  top: 5px;
  left: 5px;
}
.path_name {
  position: fixed;
  top: 40px;
  left: 5px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  width: 140px;
}
.path_size {
  position: fixed;
  top: 60px;
  left: 5px;
}
#info {
  position: fixed;
  bottom: 10px;
  left: 10px;
  text-shadow:
    1px 1px 0 rgba(255, 255, 255, 0.6),
    -1px 1px 0 rgba(255, 255, 255, 0.6),
    1px -1px 0 rgba(255, 255, 255, 0.6),
    -1px -1px 0 rgba(255, 255, 255, 0.6);
}
#repo,
#website {
  color: black;
  font-weight: bold;
}
#website {
  position: fixed;
  left: 120px;
}
</style>
