===description===
constant created by define() inside an if (! defined()) guard is indexed and resolves
===file===
<?php
if (! defined('MY_GUARDED_CONST')) {
    define('MY_GUARDED_CONST', 42);
}

function test(): void
{
    echo MY_GUARDED_CONST;
}
===expect===
