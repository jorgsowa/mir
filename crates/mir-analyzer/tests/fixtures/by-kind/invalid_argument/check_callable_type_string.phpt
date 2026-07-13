===description===
Check callable type string
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param callable(int,int):int $_p
 */
function f(callable $_p): void {}

f("strcmp");
===expect===
InvalidArgument@7:2-7:10: Argument $_p of f() expects 'callable whose parameter #1 accepts int', got 'callable whose parameter #1 only accepts string'
