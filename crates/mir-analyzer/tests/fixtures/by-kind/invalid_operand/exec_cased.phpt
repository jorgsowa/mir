===description===
Exec cased
===file===
<?php
sHeLl_EXeC("rm -rf");
===expect===
ForbiddenCode@2:1-2:21: Use of sHeLl_EXeC is forbidden
WrongCaseFunction@2:1-2:11: Function name 'sHeLl_EXeC' has incorrect casing; use 'shell_exec'
