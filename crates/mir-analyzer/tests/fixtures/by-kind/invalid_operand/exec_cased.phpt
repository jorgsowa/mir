===description===
Exec cased
===file===
<?php
sHeLl_EXeC("rm -rf");
//<^^^^^^^^^^^^^^^^^^^^ ForbiddenCode: Use of sHeLl_EXeC is forbidden
//<^^^^^^^^^^ WrongCaseFunction: Function name 'sHeLl_EXeC' has incorrect casing; use 'shell_exec'
===expect===
