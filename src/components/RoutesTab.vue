<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RefreshCw, RotateCcw } from 'lucide-vue-next'
import { loadSettings, settings } from '../lib/api'
import { t } from '../lib/i18n'
import { createSettingsDraft, provideSettingsDraft } from '../composables/useSettingsDraft'
import { useAutoSave } from '../composables/useAutoSave'
import RoutesSettings from './settings/RoutesSettings.vue'
import InfoTip from './InfoTip.vue'

/**
 * 「隧道与路由」页。
 *
 * 从设置页拆出来单独成页：这一块决定哪些网段走 VPN、隧道 MTU 多大、
 * 断开时要不要清网卡地址，属于运行行为而不是界面偏好，埋在设置页中间不好找。
 *
 * 自己持有一份草稿 + 自动保存：卡片是用 provide/inject 拿草稿的，
 * 所以 provide 必须在本组件（卡片的直接父级）完成。
 *
 * 页头与「客户端」「连接配置」「网络诊断」同款：标题在左、操作在右。
 * 「恢复默认」属于卡片内容，动作仍留在卡片里，这里只转发。
 */

const draft = createSettingsDraft()
provideSettingsDraft(draft)

const { autoState, autoText, persist, fill, markBaseline } = useAutoSave(draft, 'settings.autoSaveHint')

const routesCard = ref<InstanceType<typeof RoutesSettings> | null>(null)

onMounted(async () => {
  await loadSettings()
  if (settings.value) {
    fill(settings.value)
    // 基准值要在回填之后再取，否则第一次改动会和空指纹比较
    markBaseline(settings.value)
  }
})
</script>

<template>
  <section class="pane">
    <div class="pane-scroll">
      <header class="head">
        <div class="title-line">
          <h1>{{ t('settings.routesTitle') }}</h1>
          <InfoTip :text="t('settings.routesIntro')" />
        </div>
        <button class="ghost" @click="routesCard?.reset()">
          <RotateCcw :size="14" /> {{ t('settings.restoreDefaults') }}
        </button>
      </header>

      <RoutesSettings ref="routesCard" />
    </div>

    <div class="pane-foot">
      <span class="auto-status" :class="autoState">
        <i class="dot" />
        {{ autoText }}
      </span>
      <!-- 写盘失败时给一个明确的重试入口：改别处也能重试，但那太隐蔽了 -->
      <button v-if="autoState === 'error'" class="ghost" @click="persist">
        <RefreshCw :size="13" /> {{ t('settings.retrySave') }}
      </button>
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
</style>
