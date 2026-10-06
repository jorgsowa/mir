===description===
Regression (laravel/framework): a constant read guarded by `defined('ARTISAN_BINARY')`
is safe. mir now honors the defined() guard and no longer emits UndefinedConstant
inside the guarded branch.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function binary(): string {
    if (defined('ARTISAN_BINARY')) {
        return ARTISAN_BINARY;
    }
    return 'artisan';
}
