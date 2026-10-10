# Whether the private test server answers. The address is read from a local file that is
# never committed: Downloads\claude-loader-probe\server.txt containing ip:port.
$f = "$HOME\Downloads\claude-loader-probe\server.txt"
if (-not (Test-Path $f)) { 'no server.txt'; return }
$a = (gc $f -First 1).Trim(); $ip, $port = $a -split ':'
$u = New-Object Net.Sockets.UdpClient
$u.Client.ReceiveTimeout = 2500
# RakNet unconnected ping: id, time, offline-message magic, client guid.
$magic = [byte[]](0x00,0xff,0xff,0x00,0xfe,0xfe,0xfe,0xfe,0xfd,0xfd,0xfd,0xfd,0x12,0x34,0x56,0x78)
$pkt = [byte[]](,0x01) + [BitConverter]::GetBytes([long]1234) + $magic + [BitConverter]::GetBytes([long]99)
[void]$u.Send($pkt, $pkt.Length, $ip, [int]$port)
$state = ''
try { $ep = New-Object Net.IPEndPoint([Net.IPAddress]::Any, 0); $r = $u.Receive([ref]$ep); $state = 'LISTENING (reply ' + $r.Length + ' bytes, first 0x' + $r[0].ToString('x2') + ')' }
catch { $c = "$($_.Exception.InnerException.SocketErrorCode)"; if ($c -eq 'ConnectionReset') { $state = 'not listening yet (port closed)' } elseif ($c -eq 'TimedOut') { $state = 'no answer: listening without ping reply, or filtered' } else { $state = "error $c" } }
$u.Close()
$t = New-Object Net.Sockets.TcpClient; $ar = $t.BeginConnect($ip, 22, $null, $null); $ssh = ($ar.AsyncWaitHandle.WaitOne(2500) -and $t.Connected); $t.Close()
(Get-Date -Format 'HH:mm:ss') + " game udp ${port}: $state | ssh tcp 22 open=$ssh"
