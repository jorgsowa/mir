===description===
Laravel FP (laravel/framework): a constant read guarded by `defined('ARTISAN_BINARY')`
is safe, but mir does not honor the defined() guard and emits UndefinedConstant.
Ignored pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
function binary(): string {
    if (defined('ARTISAN_BINARY')) {
        return ARTISAN_BINARY;
    }
    return 'artisan';
}
===expect===
