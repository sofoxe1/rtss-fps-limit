$Acl = Get-Acl "C:\Program Files (x86)\RivaTuner Statistics Server\Profiles"
$Ar = New-Object system.security.accesscontrol.filesystemaccessrule("$username$","Write","ContainerInherit,ObjectInherit", "None", "Allow")
$Acl.SetAccessRule($Ar)
Set-Acl "C:\Program Files (x86)\RivaTuner Statistics Server\Profiles" $Acl