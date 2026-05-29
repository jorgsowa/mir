===description===
does not mark namespaced fn referenced via global string
===file===
<?php
namespace App;

function helper(): void {}

// 'helper' resolves as \helper (global), NOT \App\helper
call_user_func('helper');
===expect===
UnusedFunction@4:0-4:26: Function helper() is never called
