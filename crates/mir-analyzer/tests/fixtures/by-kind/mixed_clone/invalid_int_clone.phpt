===description===
Invalid int clone
===file===
<?php
$a = 5;
clone $a;
===expect===
InvalidClone@3:1-3:9: cannot clone non-object 5
