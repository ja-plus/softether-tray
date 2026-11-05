<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Bell, Minimize2, Play, PlugZap, Rocket, Square } from 'lucide-vue-next'
import SelectInput, { type SelectOption } from '../SelectInput.vue'
import { overview, showToast } from '../../lib/api'
import { t } from '../../lib/i18n'
import { useSettingsDraft } from '../../composables/useSettingsDraft'

const draft = useSettingsDraft()

const autostart = ref(false)
const error = ref('')

const accountOptions = computed<SelectOption<string | null>[]>(() => [
  { label: t('settings.noAccount'), value: null },
  ...(overview.value?.accounts ?? []).map((account) => ({ label: account.name, value: account.name })),
])

onMounted(async () => {
  try {
    autostart.value = await invoke<boolean>('autostart_enabled')
  } catch (err) {
    error.value = String(err)
  }
})

/**
 * 开机自启是系统状态，不进配置文件，所以不跟着 draft 走：
 * 点一下就地调后端，失败直接把开关拨回去。
 */
async function toggleAutostart() {
  try {
    autostart.value = await invoke<boolean>('set_autostart', { enabled: autostart.value })
    showToast(autostart.value ? t('settings.autostartOn') : t('settings.autostartOff'))
  } catch (err) {
    error.value = String(err)
    autostart.value = !autostart.value
  }
}
</script>

<template>
  <div class="card block defaults">
    <section class="group">
      <h2>{{ t('settings.defaults') }}</h2>
      <div class="field profile-field">
        <label>{{ t('settings.activeAccount') }}</label>
        <SelectInput v-model="draft.activeAccount" :options="accountOptions" />
      </div>
    </section>

    <div class="divider" aria-hidden="true" />

    <section class="group">
      <h3 class="group-title">{{ t('settings.startupSection') }}</h3>
      <div class="option-list">
        <label class="option-row" :class="{ on: draft.routes.autoConnect }">
          <span class="option-icon"><PlugZap :size="15" /></span>
          <span class="option-text">{{ t('settings.autoConnect') }}</span>
          <span class="toggle">
            <input v-model="draft.routes.autoConnect" type="checkbox" />
            <span class="track"><span class="knob" /></span>
          </span>
        </label>
        <label class="option-row" :class="{ on: draft.minimizeOnClose }">
          <span class="option-icon"><Minimize2 :size="15" /></span>
          <span class="option-text">{{ t('settings.minimizeOnClose') }}</span>
          <span class="toggle">
            <input v-model="draft.minimizeOnClose" type="checkbox" />
            <span class="track"><span class="knob" /></span>
          </span>
        </label>
        <label class="option-row" :class="{ on: draft.notifications }">
          <span class="option-icon"><Bell :size="15" /></span>
          <span class="option-text">{{ t('settings.notifications') }}</span>
          <span class="toggle">
            <input v-model="draft.notifications" type="checkbox" />
            <span class="track"><span class="knob" /></span>
          </span>
        </label>
        <label class="option-row" :class="{ on: autostart }">
          <span class="option-icon"><Rocket :size="15" /></span>
          <span class="option-text">{{ t('settings.autostart') }}</span>
          <span class="toggle">
            <input v-model="autostart" type="checkbox" @change="toggleAutostart" />
            <span class="track"><span class="knob" /></span>
          </span>
        </label>
      </div>
    </section>

    <div class="divider" aria-hidden="true" />

    <section class="group">
      <h3 class="group-title">{{ t('settings.clientServiceSection') }}</h3>
      <div class="option-list">
        <label class="option-row" :class="{ on: draft.startClientOnLaunch }">
          <span class="option-icon"><Play :size="15" /></span>
          <span class="option-text">{{ t('settings.startClient') }}</span>
          <span class="toggle">
            <input v-model="draft.startClientOnLaunch" type="checkbox" />
            <span class="track"><span class="knob" /></span>
          </span>
        </label>
        <label class="option-row" :class="{ on: draft.stopClientOnExit }">
          <span class="option-icon"><Square :size="15" /></span>
          <span class="option-text">{{ t('settings.stopClient') }}</span>
          <span class="toggle">
            <input v-model="draft.stopClientOnExit" type="checkbox" />
            <span class="track"><span class="knob" /></span>
          </span>
        </label>
      </div>
      <p class="hint">{{ t('settings.clientHint') }}</p>
    </section>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.defaults {
  gap: 14px;
}

.group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-title {
  font-weight: 500;
  color: var(--text);
}

/* 下拉框不必拉满整张卡，选配置本来就是一栏的事 */
.profile-field {
  max-width: 360px;
}

/* 满宽分隔线：卡片左右内边距是 16px，用负边距让它贯通 */
.divider {
  height: 0.5px;
  margin: 2px -16px;
  background: var(--line);
}

.option-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* 每行是一个可点的"设置条"：图标 + 文案 + 开关，悬停时整行浅底提示可点 */
.option-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 5px 8px;
  border-radius: var(--radius-ctl);
  cursor: pointer;
  transition: background 0.12s ease;
}

.option-row:hover {
  background: var(--panel-muted);
}

/* 图标是个小方块：关着时用弱色，开着时点亮成强调色，状态不必只靠开关 */
.option-icon {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 0.5px solid var(--line);
  border-radius: var(--radius-ctl);
  background: var(--panel-muted);
  color: var(--text-hint);
  transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
}

.option-row.on .option-icon {
  border-color: var(--accent-line);
  background: var(--accent-soft);
  color: var(--accent-ink);
}

.option-text {
  flex: 1;
  min-width: 0;
  color: var(--text);
  line-height: 1.45;
  /* 长句（尤其英文）在窄窗口里要能断行，而不是把开关挤出去 */
  overflow-wrap: anywhere;
}

/* ── 拨动开关 ─────────────────────────────────────────────
   原生 checkbox 盖在轨道上但完全透明：保留键盘焦点与可点区域，
   外观则自己画，三套风格都吃变量、不用为每种皮肤重做一遍。 */
.toggle {
  position: relative;
  flex: none;
  display: inline-flex;
  width: 38px;
  height: 20px;
}

.toggle input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  margin: 0;
  opacity: 0;
  cursor: pointer;
}

.track {
  display: flex;
  align-items: center;
  width: 100%;
  height: 100%;
  padding: 2px;
  border: 0.5px solid var(--off-line);
  border-radius: 999px;
  background: var(--off-bg);
  transition: background 0.16s ease, border-color 0.16s ease;
}

.knob {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
  transition: transform 0.16s ease;
}

.toggle input:checked + .track {
  border-color: var(--accent);
  background: var(--accent);
}

/* 38 宽 - 1 描边 - 4 内边距 - 14 圆点 = 19，滑到底后两侧留白对称 */
.toggle input:checked + .track .knob {
  transform: translateX(19px);
}

.toggle input:focus-visible + .track {
  box-shadow: 0 0 0 3px var(--focus-ring);
}
</style>
