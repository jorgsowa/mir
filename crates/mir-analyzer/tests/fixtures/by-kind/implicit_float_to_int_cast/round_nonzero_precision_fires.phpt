===description===
round() with a non-zero precision may return a non-integral value, so
ImplicitFloatToIntCast still fires when the result is passed to an int param.

===file===
<?php
function takes_int(int $n): void { echo $n; }

takes_int(round(3.14159, 2));
//        ^^^^^^^^^^^^^^^^^ ImplicitFloatToIntCast: Implicit cast from float to int truncates the fractional part

===expect===
