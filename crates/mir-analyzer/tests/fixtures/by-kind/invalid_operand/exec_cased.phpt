===description===
Exec cased
===config===
<mir>
  <forbiddenFunctions>
    <function name="shell_exec"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
sHeLl_EXeC("rm -rf");
//<^^^^^^^^^^^^^^^^^^^^ ForbiddenCode: Use of sHeLl_EXeC is forbidden
//<^^^^^^^^^^ WrongCaseFunction: Function name 'sHeLl_EXeC' has incorrect casing; use 'shell_exec'
===expect===
