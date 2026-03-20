# ---
# name: powershell_system_monitor
# description: PowerShell 系统监控器
# tags: [powershell, system, monitor]
# command_template: powershell -File {filepath} -Interval {seconds}
# args:
#   seconds:
#     type: integer
#     description: 监控间隔秒数
#     required: true
# ---
Write-Host "PowerShell System Monitor"
