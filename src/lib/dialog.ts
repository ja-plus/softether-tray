import { createVNode, render, type Component } from 'vue'
import AccountDialog from '../components/AccountDialog.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import NameDialog from '../components/NameDialog.vue'

/**
 * 弹窗的函数式调用入口。
 *
 * 以前弹窗靠模板里的 `v-if` + 事件回调开关，每个调用点都要养一个
 * "待确认"状态（pendingXxx），问完还得手动清。这里统一改成：
 *
 *   const ok = await confirmDialog({ ... })
 *   const saved = await openAccountDialog(name)
 *
 * 弹窗本体保持独立组件（ConfirmDialog / AccountDialog，外壳都是 Arco Modal），
 * 挂载和卸载由本模块代劳：组件在关窗动画播完后发 `close` 带上结果，
 * 这里才卸载并兑现 Promise——提前卸载会把退场动画掐掉。
 */

export interface ConfirmOptions {
  title: string
  body: string
  /** 确认按钮文案；不传就用「确定」 */
  confirmText?: string
  /** 取消按钮文案；不传就用「取消」 */
  cancelText?: string
  /** true 时确认按钮用危险色：会中断当前动作的操作都该是红的 */
  danger?: boolean
}

/** 通用二次确认。点确认 → true；取消/点遮罩/ESC/关闭 → false */
export function confirmDialog(options: ConfirmOptions): Promise<boolean> {
  return mountDialog(ConfirmDialog, { ...options })
}

/** 账号表单弹窗（name=null 为新建）。保存成功 → true；取消 → false */
export function openAccountDialog(name: string | null): Promise<boolean> {
  return mountDialog(AccountDialog, { name })
}

export interface NameDialogOptions {
  title: string
  label?: string
  placeholder?: string
  hint?: string
  /** 打开时预填的值（通常是当前可用的建议名） */
  initialValue?: string
  /** 确认按钮文案；不传就用「创建」 */
  confirmText?: string
  /** 点确认时调用；返回 false 表示失败，小窗保持打开供修改后重试 */
  onSubmit: (value: string) => Promise<boolean>
}

/**
 * 单字段命名弹窗：新建连接配置里的「创建适配器」和适配器页的「创建」共用。
 *
 * 成功关闭 → 提交的名字；取消/关闭 → null。
 */
export function nameDialog(options: NameDialogOptions): Promise<string | null> {
  return mountDialog<string | null>(NameDialog, { ...options })
}

/** 把弹窗组件挂到 body 上，等它的 close 事件拿到结果后卸载 */
function mountDialog<T>(component: Component, props: Record<string, unknown>): Promise<T> {
  return new Promise((resolve) => {
    const container = document.createElement('div')
    document.body.appendChild(container)
    const vnode = createVNode(component, {
      ...props,
      onClose: (result: T) => {
        render(null, container)
        container.remove()
        resolve(result)
      },
    })
    render(vnode, container)
  })
}
