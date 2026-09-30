===description===
Exec
===file===
<?php
shell_exec("rm -rf");
//<^^^^^^^^^^^^^^^^^^^^ ForbiddenCode: Use of shell_exec is forbidden
===expect===
