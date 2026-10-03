===description===
does not report function called via call user func
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
function helper(): void {}

call_user_func('helper');
===expect===
