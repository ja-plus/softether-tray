#!/usr/bin/env bash
# 安装 sudoers 白名单，让 softether-tray 免密启停 SoftEther 客户端。
#
# 只放开 vpnclient start / stop 两个参数组合，不放开任意命令。
# 用法：sudo bash Tools/install-sudoers.sh [/path/to/vpnclient]
# 卸载：sudo bash Tools/install-sudoers.sh --remove
set -euo pipefail

RULE_FILE=/etc/sudoers.d/softether-tray

if [[ "${1:-}" == "--remove" ]]; then
  rm -f "$RULE_FILE"
  echo "已移除 $RULE_FILE"
  exit 0
fi

# 路径优先用命令行参数；没给就在常见安装位置里找。
# 这里刻意不写死任何人的家目录——脚本是给别人用的。
VPNCLIENT="${1:-}"

if [[ -z "$VPNCLIENT" ]]; then
  # sudo 会把 HOME 换成 /root，用 SUDO_USER 找回真正调用者的家目录
  CALLER_HOME="$(getent passwd "${SUDO_USER:-${USER:-root}}" | cut -d: -f6 2>/dev/null || true)"
  candidates=()
  if [[ -n "$CALLER_HOME" ]]; then
    candidates+=("$CALLER_HOME/Downloads/vpnclient/vpnclient" "$CALLER_HOME/vpnclient/vpnclient")
  fi
  candidates+=(/usr/local/vpnclient/vpnclient /opt/vpnclient/vpnclient /usr/lib/softether-tray/client/vpnclient)
  for candidate in "${candidates[@]}"; do
    if [[ -x "$candidate" ]]; then
      VPNCLIENT="$candidate"
      break
    fi
  done
fi

if [[ -n "$VPNCLIENT" ]]; then
  VPNCLIENT="$(readlink -f "$VPNCLIENT" 2>/dev/null || true)"
fi

if [[ -z "$VPNCLIENT" || ! -x "$VPNCLIENT" ]]; then
  echo "找不到 vpnclient，请把它的完整路径作为参数传进来：" >&2
  echo "  sudo bash Tools/install-sudoers.sh /path/to/vpnclient" >&2
  exit 1
fi

# 用 visudo -c 校验后再落盘，避免写坏 sudoers 把 sudo 锁死
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT

cat > "$TMP" <<EOF
# softether-tray: 允许免密启停 SoftEther VPN 客户端（仅这两条命令）
Cmnd_Alias SOFTETHER_TRAY = $VPNCLIENT start, $VPNCLIENT stop
ALL ALL=(root) NOPASSWD: SOFTETHER_TRAY
EOF

if ! visudo -c -f "$TMP" >/dev/null 2>&1; then
  echo "sudoers 语法校验失败，未做任何修改" >&2
  exit 1
fi

install -m 0440 -o root -g root "$TMP" "$RULE_FILE"
echo "已安装 $RULE_FILE"
echo
echo "规则内容："
sed 's/^/  /' "$RULE_FILE"
echo
echo "验证：sudo -n $VPNCLIENT stop 应当不再询问密码（服务未运行时也无妨）"
