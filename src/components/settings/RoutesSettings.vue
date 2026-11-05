<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { Plus, X } from 'lucide-vue-next'
import NumberInput from '../NumberInput.vue'
import InfoTip from '../InfoTip.vue'
import { t } from '../../lib/i18n'
import { DEFAULT_MTU, DEFAULT_SUBNETS, MTU_RANGE, useSettingsDraft } from '../../composables/useSettingsDraft'

const draft = useSettingsDraft()

const newSubnet = ref('')
const error = ref('')

/** DHCP 清理开关的说明由两段词条拼成，中间夹着命令名；悬停提示里按纯文本展示 */
const dhcpHint = computed(
  () => `${t('settings.dhcpHintPrefix')}dhclient -1${t('settings.dhcpHintMid')}`,
)

/** 只收 IPv4 CIDR：IPv6 走隧道要靠路由表另行配置，这里不假装支持 */
const CIDR = /^\d{1,3}(\.\d{1,3}){3}\/\d{1,2}$/

function addSubnet() {
  const value = newSubnet.value.trim()
  if (!value) return
  if (!CIDR.test(value)) {
    error.value = t('settings.subnetError', { value })
    return
  }
  if (!draft.routes.subnets.includes(value)) draft.routes.subnets.push(value)
  newSubnet.value = ''
  error.value = ''
}

function removeSubnet(value: string) {
  clearPendingRemove()
  draft.routes.subnets = draft.routes.subnets.filter((item) => item !== value)
}

/**
 * 待确认删除的网段。
 *
 * 删除是不可撤销的写盘动作，按钮又只有一个小叉，误点代价不小。
 * 单击先把叉变成红色的「双击删除」提示，再点一下同一个叉才真的删——
 * 慢速两次点击也能删除，不依赖系统双击间隔，同时第一步就把意图说清楚。
 */
const pendingRemove = ref<string | null>(null)
let pendingTimer: number | undefined

function clearPendingRemove() {
  if (pendingTimer !== undefined) window.clearTimeout(pendingTimer)
  pendingTimer = undefined
  pendingRemove.value = null
}

/** 第一下点亮删除提示，第二下（或双击的第二下）才真正删除 */
function requestRemove(value: string) {
  if (pendingRemove.value === value) {
    removeSubnet(value)
    return
  }
  clearPendingRemove()
  pendingRemove.value = value
  pendingTimer = window.setTimeout(clearPendingRemove, 1600)
}

onBeforeUnmount(clearPendingRemove)

/** 「恢复默认」把三段私网 + MTU 一起还原；dhcp 也归位，避免只还原一半 */
function resetRoutes() {
  clearPendingRemove()
  draft.routes.subnets = [...DEFAULT_SUBNETS]
  draft.routes.mtu = DEFAULT_MTU
  draft.routes.dhcp = false
}

// 页面头部的「恢复默认」按钮在卡片之外，通过暴露的方法叫进来
defineExpose({ reset: resetRoutes })
</script>

<template>
  <div class="card block routes">
    <section class="group">
      <div class="group-head">
        <h3 class="group-title">{{ t('settings.subnetSection') }}</h3>
        <span class="count" :class="{ empty: !draft.routes.subnets.length }">
          {{ draft.routes.subnets.length }}
        </span>
        <span class="hint group-hint">{{ t('settings.subnetSectionHint') }}</span>
      </div>

      <div v-if="draft.routes.subnets.length" class="subnet-grid">
        <div v-for="subnet in draft.routes.subnets" :key="subnet" class="subnet">
          <span class="subnet-mark" aria-hidden="true" />
          <span class="mono subnet-addr" :title="subnet">{{ subnet }}</span>
          <button
            class="subnet-remove"
            :class="{ pending: pendingRemove === subnet }"
            type="button"
            :aria-label="t('settings.removeSubnetHint')"
            :title="t('settings.removeSubnetHint')"
            @click="requestRemove(subnet)"
          >
            <X :size="13" />
            <span v-if="pendingRemove === subnet">{{ t('settings.removeSubnetHint') }}</span>
          </button>
        </div>
      </div>
      <div v-else class="empty">
        {{ t('settings.noSubnets') }}
      </div>

      <div class="add-row">
        <input
          v-model="newSubnet"
          class="add-input"
          type="text"
          :placeholder="t('settings.subnetPlaceholder')"
          spellcheck="false"
          @keyup.enter="addSubnet"
        />
        <!-- 空输入时置灰：点下去没反应会让人以为按钮坏了，先明确"还不可点" -->
        <button class="primary" :disabled="!newSubnet.trim()" @click="addSubnet">
          <Plus :size="14" />
          {{ t('settings.add') }}
        </button>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
    </section>

    <div class="divider" aria-hidden="true" />

    <section class="group">
      <div class="group-head">
        <h3 class="group-title">{{ t('settings.tunnelSection') }}</h3>
      </div>

      <div class="tunnel-grid">
        <div class="field mtu-field">
          <div class="label-row">
            <label>{{ t('settings.mtu') }}</label>
            <InfoTip :text="t('settings.mtuHint')" />
          </div>
          <div class="mtu-control">
            <div class="mtu-input">
              <NumberInput v-model="draft.routes.mtu" :min="MTU_RANGE.min" :max="MTU_RANGE.max" :step="10" />
            </div>
            <span class="hint">{{ t('settings.mtuUnit') }}</span>
          </div>
        </div>

        <div class="option">
          <input id="routes-dhcp-clear" v-model="draft.routes.dhcp" type="checkbox" />
          <label class="option-text" for="routes-dhcp-clear">
            <strong>{{ t('settings.flushAddr') }}</strong>
          </label>
          <span class="option-tip">
            <InfoTip :text="dhcpHint" />
          </span>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.routes {
  gap: 14px;
}

.group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.group-head {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.group-title {
  font-weight: 500;
  color: var(--text);
}

.count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 999px;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  background: var(--accent-soft);
  color: var(--accent-ink);
  border: 0.5px solid var(--accent-line);
}

.count.empty {
  background: var(--off-bg);
  color: var(--text-soft);
  border-color: var(--off-line);
}

/* 说明文字推到最右，和左侧标题、计数分开 */
.group-hint {
  margin-left: auto;
  text-align: right;
}

.subnet-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
  gap: 8px;
}

.subnet {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  padding: 5px 6px 5px 11px;
  border: 0.5px solid var(--accent-line);
  border-radius: var(--radius-ctl);
  background: var(--accent-soft);
  color: var(--accent-ink);
  transition: border-color 0.12s ease;
}

.subnet:hover {
  border-color: var(--accent);
}

/* 小圆点让每个地址形成一个"路由条目"的观感，也方便快速数条数 */
.subnet-mark {
  flex: none;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.55;
}

.subnet-addr {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.subnet-remove {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  width: 22px;
  height: 22px;
  padding: 0;
  border: 0.5px solid transparent;
  border-radius: 999px;
  background: transparent;
  color: inherit;
  font-size: 11px;
  line-height: 1;
  white-space: nowrap;
  opacity: 0.5;
  transition: background 0.12s ease, color 0.12s ease, opacity 0.12s ease;
}

.subnet-remove:hover {
  background: var(--danger-soft);
  color: var(--danger);
  opacity: 1;
}

/* 单击后撑开成「× 双击删除」：叉本身变红并带一圈描边，
   让用户一眼知道还得再点一下，而不是以为按钮点不动 */
.subnet-remove.pending {
  width: auto;
  padding: 0 7px 0 5px;
  border-color: var(--danger-line);
  background: var(--danger-soft);
  color: var(--danger);
  opacity: 1;
  animation: remove-arm 0.28s ease-out;
}

@keyframes remove-arm {
  0% {
    transform: scale(0.86);
  }
  60% {
    transform: scale(1.06);
  }
  100% {
    transform: scale(1);
  }
}

.empty {
  padding: 14px;
  border: 0.5px dashed var(--line-strong);
  border-radius: var(--radius-ctl);
  color: var(--text-hint);
  font-size: 12px;
  text-align: center;
}

.add-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.add-input {
  flex: 1 1 auto;
  min-width: 0;
  max-width: 300px;
}

/* 满宽分隔线：卡片左右内边距是 16px，用负边距让它贯通 */
.divider {
  height: 0.5px;
  margin: 2px -16px;
  background: var(--line);
}

.tunnel-grid {
  display: grid;
  grid-template-columns: minmax(180px, 220px) minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}

/* 字段标签和问号同一行，问号只负责悬停展开说明 */
.label-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.mtu-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mtu-input {
  flex: none;
  width: 140px;
}

/* 开关做成一块可点的说明条；解释长文收进右侧问号，悬停才展开 */
.option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 0.5px solid var(--line);
  border-radius: var(--radius-ctl);
  background: var(--panel-muted);
}

.option:hover {
  border-color: var(--line-strong);
}

.option input {
  flex: none;
  margin: 0;
}

.option-text {
  flex: 1;
  min-width: 0;
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
}

.option-text strong {
  font-weight: 500;
}

.option-tip {
  flex: none;
  display: inline-flex;
}

@media (max-width: 560px) {
  .tunnel-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
