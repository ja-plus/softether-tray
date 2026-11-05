<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Loader2, Power, Trash2 } from 'lucide-vue-next'
import { confirmDialog, nameDialog } from '../../lib/dialog'
import { DEFAULT_NIC_NAME, nextNicName, overview, refresh, showToast, type NicInfo } from '../../lib/api'
import { t } from '../../lib/i18n'

/**
 * 虚拟网络适配器管理。
 *
 * 为什么值得单独一张卡：适配器是**连接的前提**——没有虚拟网卡，新建的连接配置
 * 根本没有网卡可绑，连上也无处落地。以前只能靠新建配置弹窗里的"创建"按钮，
 * 用户不新建配置就永远看不到它，删/停更没入口。
 */
const nics = computed<NicInfo[]>(() => overview.value?.nics ?? [])

/**
 * 适配器清单是 vpncmd 读出来的，客户端没跑就一定是空的。
 *
 * 这一页以前埋在设置里最底下、没什么人单独进；现在它是左侧一级入口，
 * 空列表如果只说"还没有任何适配器"，会让人以为自己的网卡丢了，
 * 实际上只是没启动客户端。所以这里分两种情况说。
 */
const serviceReady = computed(
  () => !!overview.value?.serviceRunning && !!overview.value?.vpncmdReady,
)

/** 正在操作的那张网卡（禁用/启用/删除），用于只锁对应行 */
const pending = ref<string | null>(null)

/** 只替换 nics 字段。overview 为 null 时（首屏还没到）什么都不做，别把整个对象弄坏 */
function applyNics(list: NicInfo[]) {
  if (!overview.value) return
  overview.value = { ...overview.value, nics: list }
}

onMounted(async () => {
  // 设置页没有轮询，进页面时主动读一次，别拿启动时的旧清单
  try {
    await refresh()
  } catch {
    // 读不到列表时保留默认值，创建失败会由后端给出真实原因
  }
})

/** 点右上角「创建」才弹命名小窗；与新建连接配置共用同一个弹窗 */
async function create() {
  await nameDialog({
    title: t('nic.newTitle'),
    label: t('nic.nameLabel'),
    placeholder: t('nic.newPlaceholder'),
    hint: t('nic.nameHint'),
    initialValue: nextNicName(DEFAULT_NIC_NAME, nics.value.map((nic) => nic.name)),
    onSubmit: async (typed) => {
      // 手动输入的名字也可能撞名（列表可能没刷新）：撞了就按顺序改成 base2、base3…
      const taken = nics.value.map((nic) => nic.name)
      const name = taken.some((item) => item.trim().toLowerCase() === typed.toLowerCase())
        ? nextNicName(typed, taken)
        : typed
      try {
        applyNics(await invoke<NicInfo[]>('create_nic', { name }))
        showToast(t('nic.created', { name }))
        return true
      } catch (error) {
        showToast(String(error), true)
        return false
      }
    },
  })
}

async function toggle(nic: NicInfo) {
  if (pending.value) return
  pending.value = nic.name
  try {
    applyNics(
      await invoke<NicInfo[]>('set_nic_enabled', { name: nic.name, enabled: !nic.enabled }),
    )
    showToast(nic.enabled ? t('nic.disabled', { name: nic.name }) : t('nic.enabled', { name: nic.name }))
  } catch (error) {
    showToast(String(error), true)
  } finally {
    pending.value = null
  }
}

async function askDelete(name: string) {
  const ok = await confirmDialog({
    title: t('nic.deleteTitle'),
    body: t('nic.deleteBody', { name }),
    confirmText: t('common.delete'),
    danger: true,
  })
  if (!ok) return
  pending.value = name
  try {
    applyNics(await invoke<NicInfo[]>('delete_nic', { name }))
    showToast(t('nic.deleted', { name }))
  } catch (error) {
    showToast(String(error), true)
  } finally {
    pending.value = null
  }
}

// 「新建」按钮按页面统一布局放在页头（右侧操作区），逻辑仍留在这张卡片里：
// 命名、重名处理都要看当前清单，搬出去只会让两处各拿一份 nics。
defineExpose({ create })
</script>

<template>
  <div class="card block">
    <!-- 空状态要说清后果：没有适配器时新建的连接配置绑不上网卡。
         客户端没跑时列表必然为空，这时说的是另一件事（去启动服务）。 -->
    <p v-if="!serviceReady" class="hint empty">{{ t('nic.needService') }}</p>
    <p v-else-if="!nics.length" class="hint empty">{{ t('nic.empty') }}</p>

    <ul v-else class="nic-list">
      <li v-for="nic in nics" :key="nic.name">
        <div class="nic-main">
          <div class="nic-head">
            <span class="strong">{{ nic.name }}</span>
            <span class="badge" :class="nic.enabled ? 'ok' : 'off'">{{ nic.enabled ? t('common.enabled') : t('common.disabled') }}</span>
          </div>
          <span v-if="nic.mac" class="mono hint">{{ nic.mac }}</span>
        </div>
        <div class="nic-actions">
          <button class="ghost small" :disabled="!!pending" @click="toggle(nic)">
            <Loader2 v-if="pending === nic.name" :size="13" class="spin" />
            <Power v-else :size="13" />
            {{ nic.enabled ? t('nic.disable') : t('nic.enable') }}
          </button>
          <button
            class="danger small"
            :disabled="!!pending"
            :title="t('nic.delete')"
            @click="askDelete(nic.name)"
          >
            <Trash2 :size="13" />
            {{ t('nic.delete') }}
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.nic-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nic-list li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 0.5px solid var(--line);
  border-radius: var(--radius-ctl);
}

.nic-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.nic-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.nic-actions {
  display: flex;
  gap: 8px;
  flex: none;
}

.nic-actions button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.empty {
  margin: 0;
}

.spin {
  animation: spin 0.9s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
