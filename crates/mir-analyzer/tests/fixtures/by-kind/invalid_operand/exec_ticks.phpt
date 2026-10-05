===description===
Exec ticks
===config===
<mir>
  <forbiddenFunctions>
    <function name="shell_exec"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
`rm -rf`;
//<^^^^^^^^ ForbiddenCode: Use of shell_exec (backtick) is forbidden
===expect===
