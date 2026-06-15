===description===
Exec cased
===file===
<?php
sHeLl_EXeC("rm -rf");
===expect===
ForbiddenCode@2:0-2:20: Use of sHeLl_EXeC is forbidden
WrongCaseFunction@2:0-2:10: Function name 'sHeLl_EXeC' has incorrect casing; use 'shell_exec'
