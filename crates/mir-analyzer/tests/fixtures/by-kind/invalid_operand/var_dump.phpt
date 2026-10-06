===description===
Var dump
===config===
<mir>
  <forbiddenFunctions>
    <function name="var_dump"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
var_dump("hello");
//<^^^^^^^^^^^^^^^^^ ForbiddenCode: Use of var_dump is forbidden
