===description===
does not report function called via call user func double quoted string
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
function helper(): void {}

call_user_func("helper");
===expect===
