===description===
FN: prefix -- never checked its operand, unlike postfix --.
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
--$a;
===expect===
InvalidOperand@3:2-3:4: Operator '--' not supported between '"hello"' and ''
