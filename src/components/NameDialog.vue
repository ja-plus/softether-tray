<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { Modal as AModal } from '@arco-design/web-vue'
import '@arco-design/web-vue/es/modal/style/css.js'
import { t } from '../lib/i18n'

/**
 * 单字段命名弹窗（Arco Modal 外壳）。
 *
 * 「新建连接配置」和「虚拟网络适配器」页共用这一个：点创建按钮才弹出，
 * 输入名字并确认。真正干活的是调用方传进来的 onSubmit——它返回 false
 * （比如建卡失败）时小窗保持打开，用户改个名字就能原地重试。
 *
 * 一般不在模板里挂它，走 lib/dialog.ts 的 nameDialog() 函数式调用。
 */
const props = defineProps<{
  title: string
  label?: string
  placeholder?: string
  hint?: string
  initialValue?: string
  /** 确认按钮文案；不传就用「创建」 */
  confirmText?: string
  onSubmit: (value: string) => Promise<boolean>
}>()

const emit = defineEmits<{ close: [value: string | null] }>()

const visible = ref(true)
const value = ref(props.initialValue ?? '')
/** 输入框挂到 DOM 后立刻聚焦，弹窗一出现就能直接打字 */
const input = ref<HTMLInputElement | null>(null)
/** 提交中的忙碌态：锁住按钮并改文案，避免连点建出两块网卡 */
const pending = ref(false)
/** 提交成功后才把输入值作为结果带出去 */
const submitted = ref(false)

onMounted(() => nextTick(() => input.value?.focus()))

/** Modal 确定按钮的前置钩子：返回 false 保持弹窗打开 */
async function confirm(): Promise<boolean> {
  const typed = value.value.trim()
  if (!typed || pending.value) return false
  pending.value = true
  try {
    const ok = await props.onSubmit(typed)
    if (ok) submitted.value = true
    return ok
  } finally {
    pending.value = false
  }
}
</script>

<template>
  <AModal
    v-model:visible="visible"
    :title="title"
    :width="420"
    title-align="start"
    :mask-closable="false"
    :ok-text="pending ? t('common.saving') : (confirmText ?? t('common.create'))"
    :cancel-text="t('common.cancel')"
    :ok-button-props="{ disabled: !value.trim() || pending }"
    :on-before-ok="confirm"
    @close="emit('close', submitted ? value.trim() : null)"
  >
    <div class="field">
      <label v-if="label">{{ label }}</label>
      <input ref="input" v-model="value" type="text" :placeholder="placeholder" />
    </div>
    <p v-if="hint" class="hint" style="margin: 8px 0 0">{{ hint }}</p>
  </AModal>
</template>
