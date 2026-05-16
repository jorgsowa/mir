===description===
does not report function called via call user func with fqn
===file===
<?php
namespace App;

function helper(): void {}

// Explicit FQN with backslash prefix in the string
call_user_func('\App\helper');
===expect===
===ignore===
TODO
