===description===
variable accessed through variable-variable with known name should not be reported as unused

===config===
suppress=MissingReturnType,UnusedVariable
===file===
<?php
function test() {
    $varName = 'foo';
    $foo = 'bar';
    return $$varName;
}
===expect===
