===description===
floor()/ceil()/round() return float. Passing their result to an int-typed parameter
emits ImplicitFloatToIntCast (Warning) but never InvalidArgument in non-strict mode —
PHP coerces the value silently.

===config===
php_version=8.1
suppress=UnusedParam

===file===
<?php
function takes_int(int $n): void {}

$a = floor(3.7);
takes_int($a);

$b = ceil(3.1);
takes_int($b);

$c = round(3.5);
takes_int($c);

===expect===
ImplicitFloatToIntCast@5:10-5:12: Implicit cast from float to int truncates the fractional part
ImplicitFloatToIntCast@8:10-8:12: Implicit cast from float to int truncates the fractional part
ImplicitFloatToIntCast@11:10-11:12: Implicit cast from float to int truncates the fractional part
