===config===
find_dead_code=true
===file===
<?php
namespace App;

function helper(): void {}

// 'helper' resolves as \helper (global), NOT \App\helper
call_user_func('helper');
===expect===
UnusedFunction: Function helper() is never called
