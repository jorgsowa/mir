===description===
"NAN"/"INF"/"Infinity" literal strings are rejected by PHP's is_numeric(),
unlike Rust's f64 parser — arithmetic on them must still flag InvalidOperand.
===config===
suppress=UnusedVariable
===file===
<?php
$a = "NAN" + 1;
$b = "INF" * 2;
$c = "5" + 1;
===expect===
InvalidOperand@2:5-2:14: Operator '+' not supported between '"NAN"' and '1'
InvalidOperand@3:5-3:14: Operator '*' not supported between '"INF"' and '2'
