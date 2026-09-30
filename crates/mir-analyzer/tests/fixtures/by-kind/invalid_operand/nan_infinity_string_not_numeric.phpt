===description===
"NAN"/"INF"/"Infinity" literal strings are rejected by PHP's is_numeric(),
unlike Rust's f64 parser — arithmetic on them must still flag InvalidOperand.
===config===
suppress=UnusedVariable
===file===
<?php
$a = "NAN" + 1;
//   ^^^^^^^^^ InvalidOperand: Operator '+' not supported between '"NAN"' and '1'
$b = "INF" * 2;
//   ^^^^^^^^^ InvalidOperand: Operator '*' not supported between '"INF"' and '2'
$c = "5" + 1;
===expect===
