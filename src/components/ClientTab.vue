<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Loader2, Power, RefreshCw } from 'lucide-vue-next'
import {
  busy,
  loadSettings,
  overview,
  refresh,
  settings,
  startService,
  stopService,
} from '../lib/api'
import { t } from '../lib/i18n'
import { createSettingsDraft, provideSettingsDraft } from '../composables/useSettingsDraft'
import { useAutoSave } from '../composables/useAutoSave'
import ClientDirSettings from './settings/ClientDirSettings.vue'
import InfoTip from './InfoTip.vue'

/**
 * 「SoftEther 客户端」页。
 *
 * 从设置页拆出来单独成页：这一块决定的是**用哪份 vpnclient**，会停/起服务、
 * 会切目录，跟语言、样式这类偏好不是一类东西，埋在设置里最底下没人找得到。
 *
 * 自己持有一份草稿 + 自动保存：卡片是用 provide/inject 拿草稿的，
 * 所以 provide 必须在本组件（卡片的直接父级）完成。
 *
 * 页头与「连接配置」「网络诊断」同款：标题在左、操作在右，卡片排在下方。
 * 启停服务与重新扫描动作本身仍留在卡片里（扫描状态归它），这里只转发。
 */

const draft = createSettingsDraft()
provideSettingsDraft(draft)

const { autoState, autoText, persist, fill, markBaseline } = useAutoSave(draft, 'client.autoSaveHint')

const clientDir = ref<InstanceType<typeof ClientDirSettings> | null>(null)

/** 客户端服务是否在跑。启停按钮跟着它切换红/绿 */
const serviceRunning = computed(() => overview.value?.serviceRunning ?? false)
/** 启停服务进行中：按钮转圈，避免连点 */
const serviceBusy = computed(() => busy.value === 'service')
/** 扫盘进行中：扫描图标转圈并禁用，避免重复触发 */
const scanning = computed(() => clientDir.value?.scanning ?? false)

onMounted(async () => {
  await loadSettings()
  if (settings.value) {
    fill(settings.value)
    markBaseline(settings.value)
  }
  // 卡片要靠 overview 判断"服务在不在跑、实际跑的是哪个目录"，
  // 而轮询只在「连接」页开着，这里进页面主动读一次，否则显示的是启动时的旧状态。
  void refresh().catch(() => {})
})
</script>

<template>
  <section class="pane">
    <div class="pane-scroll">
      <header class="head">
        <div class="title-line">
          <h1>{{ t('client.title') }}</h1>
          <InfoTip :text="t('client.intro', { cmd: 'vpnclient' })" />
        </div>
        <div class="row actions">
          <!-- 启停服务放在客户端页：它管的就是这个客户端，
               放在连接页反而让人以为跟某条隧道绑定。 -->
          <button
            v-if="serviceRunning"
            class="danger"
            :disabled="!!busy"
            :title="t('overview.stopTooltip')"
            @click="stopService()"
          >
            <Loader2 v-if="serviceBusy" :size="14" class="spin" />
            <Power v-else :size="14" />
            {{ t('overview.stopService') }}
          </button>
          <button
            v-else
            class="success"
            :disabled="!!busy"
            :title="t('overview.startTooltip')"
            @click="startService()"
          >
            <Loader2 v-if="serviceBusy" :size="14" class="spin" />
            <Power v-else :size="14" />
            {{ t('overview.startService') }}
          </button>
          <!-- 重新扫描只留图标：文字在页头抢不过启停按钮 -->
          <button
            class="ghost icon-only"
            :disabled="scanning"
            :aria-label="t('client.rescan')"
            :title="scanning ? t('client.scanning') : t('client.rescan')"
            @click="clientDir?.scanClients()"
          >
            <RefreshCw :size="15" :class="{ spin: scanning }" />
          </button>
        </div>
      </header>

      <ClientDirSettings ref="clientDir" />
    </div>

    <div class="pane-foot">
      <span class="auto-status" :class="autoState">
        <i class="dot" />
        {{ autoText }}
      </span>
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

/* 右侧操作：启停按钮在前，扫描图标在后，图标按钮不吃多余宽度 */
.actions {
  align-items: center;
  flex: none;
}

.actions button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

/* 扫描图标转起来：不加这个动画用户会以为"没反应" */
.spin {
  animation: spin 0.9s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
