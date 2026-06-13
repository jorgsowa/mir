===description===
Regression (laravel/framework): a constant read guarded by `defined('ARTISAN_BINARY')`
is safe. mir now honors the defined() guard and no longer emits UndefinedConstant
inside the guarded branch.
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
