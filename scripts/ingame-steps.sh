# The steps of peek's in-game run, for amml's runner (docs/ingame.md in amml). The run makes a shulker box with items
# in the inventory with chat commands, opens the inventory screen and moves the pointer over it. The logs are
# `modmanager` and `peek`.
#
# A box with two or more bundles that hold items cannot be made with a command (`give` and `replaceitem` take no bundle
# contents), so the bundle pick needs that box in the world amml-test-peek: put the box in the inventory of the player
# in that world once, in the game, and the pristine copy of the world keeps it. Without such a box the bundle pick is
# not run and the run says why.

# the pointer position (fractions of the client area) where the hover line of the mod came
hover_rx=""
hover_ry=""
# the number of bundles in the box, read from the last box line
bundles_in_box=0

# Posted input that the runner does not give: WM_MOUSEMOVE at a point given in fractions of the client area, and
# WM_MOUSEWHEEL (the sign of the notches is the direction: +1 is wheel up). The messages go to the window of one PID
# with PostMessage; nothing uses SendInput or the real cursor.
write_input_helper() {
  cat > "$out_dir/post-peek-input.ps1" <<'PS_HELPER'
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][ValidateSet('move', 'wheel')][string]$Op,
  [double]$RX = 0.5,
  [double]$RY = 0.5,
  [int]$Notches = 0
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class PeekInput {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr hwnd, ref POINT point);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

  public static IntPtr Find(uint pid) {
    IntPtr best = IntPtr.Zero;
    long bestArea = 0;
    EnumWindows((hwnd, _) => {
      uint owner;
      GetWindowThreadProcessId(hwnd, out owner);
      RECT r;
      if (owner == pid && IsWindowVisible(hwnd) && GetClientRect(hwnd, out r)) {
        long area = (long)(r.Right - r.Left) * (r.Bottom - r.Top);
        if (area > bestArea) { bestArea = area; best = hwnd; }
      }
      return true;
    }, IntPtr.Zero);
    return best;
  }

  static POINT At(IntPtr hwnd, double rx, double ry) {
    RECT r;
    GetClientRect(hwnd, out r);
    POINT p;
    p.X = (int)((r.Right - r.Left) * rx);
    p.Y = (int)((r.Bottom - r.Top) * ry);
    return p;
  }

  public static void Move(IntPtr hwnd, double rx, double ry) {
    POINT p = At(hwnd, rx, ry);
    PostMessageW(hwnd, 0x0200, IntPtr.Zero, (IntPtr)((p.Y << 16) | (p.X & 0xFFFF)));
  }

  public static void Wheel(IntPtr hwnd, double rx, double ry, int notches) {
    POINT p = At(hwnd, rx, ry);
    ClientToScreen(hwnd, ref p);
    long wparam = ((long)(short)(notches * 120) << 16) & 0xFFFF0000L;
    PostMessageW(hwnd, 0x020A, (IntPtr)wparam, (IntPtr)((p.Y << 16) | (p.X & 0xFFFF)));
  }
}
'@
[PeekInput]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$hwnd = [PeekInput]::Find([uint32]$ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { Write-Host "no visible window for PID $ProcessId"; exit 1 }
if ($Op -eq 'move') { [PeekInput]::Move($hwnd, $RX, $RY) } else { [PeekInput]::Wheel($hwnd, $RX, $RY, $Notches) }
PS_HELPER
}

# Runs the helper of write_input_helper with the arguments given
input_run() {
  powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$(cygpath -w "$out_dir/post-peek-input.ps1")" "$@"
}

# Types one chat command: `/` opens the chat with the slash, then the text and Enter
chat_command() {
  local text="$1" i
  post_key 191
  sleep 1
  for (( i = 0; i < ${#text}; i++ )); do
    post_char "${text:$i:1}" > /dev/null
  done
  post_key 13
  sleep 1
}

# Moves the pointer over the grid of points that covers the hotbar and the inventory of the inventory screen, until the
# hover line of the mod shows; false when no point gave one. The first point of the row of the hotbar comes first
sweep_for_box() {
  local min="${1:-0}" rx ry
  for ry in 0.80 0.70 0.64 0.58 0.52; do
    for rx in 0.38 0.42 0.46 0.50 0.54 0.58 0.62; do
      alive || return 1
      input_run -ProcessId "$game_pid" -Op move -RX "$rx" -RY "$ry" > /dev/null || echo "posting a move failed"
      sleep 0.4
      if [ "$(count_lines peek 'peek box items=')" -gt "$min" ]; then
        hover_rx="$rx"
        hover_ry="$ry"
        return 0
      fi
    done
  done
  return 1
}

ingame_setup() {
  write_input_helper
}

ingame_play() {
  wait_line modmanager "manager enabled" 180 || true
  wait_line peek "peek linked" 120 || true
  wait_line peek "features off: none" 20 || true
  wait_line loader 'top screen name .*hud_screen' 120 || return 0
  sleep 5
  alive || return 0
  echo "--- a shulker box with items in the inventory"
  chat_command "gamemode survival"
  chat_command "setblock ~ ~2 ~ purple_shulker_box"
  chat_command "replaceitem block ~ ~2 ~ slot.container 0 stone 64"
  chat_command "replaceitem block ~ ~2 ~ slot.container 1 diamond_pickaxe 1 0"
  chat_command "replaceitem block ~ ~2 ~ slot.container 2 apple 3"
  chat_command "setblock ~ ~2 ~ air destroy"
  # the box drops from two blocks up and is picked up where the player stands
  sleep 4
  echo "--- inventory screen, pointer over the box"
  post_key 69
  sleep 2
  sweep_for_box 0 || echo "no point of the sweep gave a box line"
  shot hover
  # the draw time line comes after five seconds of drawing
  sleep 7
  bundles_in_box=$(run_lines peek | grep -E 'peek box items=' | tail -n 1 | sed -n 's/.*bundles=\([0-9]*\).*/\1/p')
  bundles_in_box="${bundles_in_box:-0}"
  if [ "$bundles_in_box" -ge 2 ]; then
    echo "--- bundle pick: shift down, wheel down, shift up"
    post_key 16 down
    sleep 1
    input_run -ProcessId "$game_pid" -Op wheel -RX "${hover_rx:-0.5}" -RY "${hover_ry:-0.7}" -Notches -1 || echo "posting the wheel failed"
    wait_line peek "peek bundle selected index=" 10 || true
    post_key 16 up
    shot bundle
  else
    echo "the box in the inventory has $bundles_in_box bundles; the bundle pick needs 2 or more (see the header of this file)"
  fi
  echo "--- close the screen with the pointer on the box"
  post_key 27
  wait_line peek "peek screen closed screen=" 15 || true
}

ingame_check() {
  local log bad name
  check peek "peek linked" "link: peek linked"
  check peek "features off: none" "features: features off: none"
  check loader 'area adapter .*"inventory\.read"' "area adapter inventory.read"
  check loader 'ui thread .*"hook": ?"ScreenView.render".*"game": ?true' "ScreenView.render runs on the game thread"
  # Posted mouse moves do not move the game's pointer in the background, so the game hovers only the slot under the
  # real pointer when the screen opens. Any hover proves the event path
  check peek "peek hover screen=[0-9]+ collection=[a-z]+ slot=[0-9]+" "hover: the event reaches the mod"
  check peek "peek hovertext contents_lines=0" "hovertext: contents lines hidden"
  info "attended rows" "the box preview, contents, counts, draw time and bundle pick are not checked in game: the pointer cannot be posted in the background"
  check peek "peek screen closed screen=[0-9]+" "closing: screen closed after Escape"
  for log in loader modmanager peek; do
    bad=$(count_lines "$log" "area withdrawn|needs @amml/modmanager")
    if [ "$bad" -eq 0 ]; then ok "no withdrawn or needs-modmanager line in the $log log"; else fail "$log log lines" "$bad withdrawn or needs-modmanager lines in the $log log"; fi
  done
  for name in hover bundle; do
    [ -f "$out_dir/$name.png" ] && info "screenshot" "$out_dir/$name.png"
  done
  return 0
}
