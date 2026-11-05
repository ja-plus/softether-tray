<script setup lang="ts">
import { computed, ref } from 'vue'
import { Plus } from 'lucide-vue-next'
import NicSettings from './settings/NicSettings.vue'
import InfoTip from './InfoTip.vue'
import { overview } from '../lib/api'
import { t } from '../lib/i18n'

/**
 * 「虚拟网络适配器」页。
 *
 * 这一页没有任何要落盘的偏好——增删启停都由后端立刻写进客户端配置，
 * 数据源是 `overview.nics`（卡片自己会在挂载时 refresh 一次），
 * 所以这里不需要草稿，也不需要底部那条自动保存状态栏。
 *
 * 页头和「连接配置」「网络诊断」保持同一套布局：标题在左、操作在右，
 * 内容卡片整体排在下方的滚动区里。
 */

const nicCard = ref<InstanceType<typeof NicSettings> | null>(null)

/** 适配器要 vpncmd 才能读写；服务没跑时「新建」无从下手，按钮置灰并说明原因 */
const serviceReady = computed(
  () => !!overview.value?.serviceRunning && !!overview.value?.vpncmdReady,
)
</script>

<template>
  <section class="pane">
    <div class="pane-scroll">
      <header class="head">
        <div class="title-line">
          <h1>{{ t('nic.title') }}</h1>
          <InfoTip :text="t('nic.intro')" />
        </div>
        <button
          class="primary"
          :disabled="!serviceReady"
          :title="serviceReady ? undefined : t('nic.needService')"
          @click="nicCard?.create()"
        >
          <Plus :size="14" />
          {{ t('nic.create') }}
        </button>
      </header>

      <NicSettings ref="nicCard" />
    </div>
  </section>
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  flex: none;
}

.title-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.head button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
</style>
