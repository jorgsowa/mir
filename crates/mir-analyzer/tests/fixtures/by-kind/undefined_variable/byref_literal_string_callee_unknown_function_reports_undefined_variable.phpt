===description===
Literal-string callee naming an unknown function cannot premark, so the argument is reported as UndefinedVariable
===file===
<?php
function d(): void {
    $fn = 'no_such_fn_xyz';
    $fn($x);
//      ^^ UndefinedVariable: Variable $x is not defined
//  ^^^^^^^ UndefinedFunction: Function no_such_fn_xyz() is not defined
}
