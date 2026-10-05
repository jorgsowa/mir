===description===
Exec
===config===
<mir>
  <forbiddenFunctions>
    <function name="shell_exec"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
shell_exec("rm -rf");
//<^^^^^^^^^^^^^^^^^^^^ ForbiddenCode: Use of shell_exec is forbidden
===expect===
