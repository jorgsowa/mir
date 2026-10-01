===description===
Literal-string callee naming an unknown function cannot premark, so the argument is reported as UndefinedVariable
===file===
<?php
function d(): void {
    $fn = 'no_such_fn_xyz';
    $fn($x);
}
===expect===
UndefinedVariable@4:8-4:10: Variable $x is not defined
