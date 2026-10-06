===description===
does not report function called via call user func with fqn
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
namespace App;

function helper(): void {}

// Explicit FQN with backslash prefix in the string
call_user_func('\App\helper');
