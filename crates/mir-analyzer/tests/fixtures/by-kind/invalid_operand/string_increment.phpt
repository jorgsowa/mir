===description===
String increment
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
$a++;
===expect===
InvalidOperand@3:0-3:2: Operator '++' not supported between '"hello"' and ''
