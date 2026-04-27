===config===
find_dead_code=true
===file===
<?php
namespace App;

function helper(): void {}

// Explicit FQN with backslash prefix in the string
call_user_func('\App\helper');
===expect===
