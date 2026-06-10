===description===
Possibly invalid bitwise not
===file===
<?php
$a = ~(rand(0, 1) ? 2 : null);
===expect===
