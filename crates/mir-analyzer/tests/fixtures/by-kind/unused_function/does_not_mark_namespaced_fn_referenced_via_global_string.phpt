===description===
does not mark namespaced fn referenced via global string
===file===
<?php
namespace App;

function helper(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function helper() is never called

// 'helper' resolves as \helper (global), NOT \App\helper
call_user_func('helper');
===expect===
