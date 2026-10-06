===description===
Invalid int clone
===file===
<?php
$a = 5;
clone $a;
//<^^^^^^^^ InvalidClone: cannot clone non-object 5
