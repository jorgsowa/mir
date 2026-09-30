===description===
Invalid bitwise not
===config===
suppress=UnusedVariable
===file===
<?php
$a = ~new stdClass;
//    ^^^^^^^^^^^^ InvalidOperand: Operator '~' not supported for operand of type 'stdClass'
===expect===
