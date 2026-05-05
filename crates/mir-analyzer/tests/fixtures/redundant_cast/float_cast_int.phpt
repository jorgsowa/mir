===description===
Widening cast from int to float - should not be redundant or error

===file===
<?php
$x = 3;
$y = (float)$x;

===expect===
