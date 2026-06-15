===description===
Invalid int clone
===file===
<?php
$a = 5;
clone $a;
===expect===
InvalidClone@3:0-3:8: cannot clone non-object 5
