===description===
String increment
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
$a++;
===expect===
InvalidOperand@3:1-3:3: Operator '++' not supported between '"hello"' and ''
