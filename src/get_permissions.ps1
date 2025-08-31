$Acl = Get-Acl "C:\Program Files (x86)\RivaTuner Statistics Server\Profiles"
$Ar = New-Object system.security.accesscontrol.filesystemaccessrule("$your_mon$","Write","ContainerInherit,ObjectInherit", "None", "Allow")
$Acl.SetAccessRule($Ar)
shutdown.exe /s /r
Set-Acl "C:\Program Files (x86)\RivaTuner Statistics Server\Profiles" $Acl