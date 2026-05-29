===description===
hrtime(true) returns int|float, not int|float|false — casting to int must not emit InvalidCast

===file===
<?php
$ns = hrtime(true);
$int = (int)$ns;

===expect===
