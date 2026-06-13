===description===
Enum wrong float
===config===
suppress=UnusedParam
===file===
<?php
namespace Ns;

/** @param 1.2|3.4|5.6 $s */
function foo($s) : void {}
foo(7.8);
===expect===
InvalidArgument@6:5-6:8: Argument $s of foo() expects 'Ns\1.2|Ns\3.4|Ns\5.6', got '7.8'
