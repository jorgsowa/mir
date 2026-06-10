===description===
Exec
===file===
<?php
shell_exec("rm -rf");
===expect===
ForbiddenCode@2:1-2:21: Use of shell_exec is forbidden
