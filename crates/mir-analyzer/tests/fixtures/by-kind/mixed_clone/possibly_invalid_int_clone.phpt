===description===
Possibly invalid int clone
===file===
<?php
$a = rand(0, 1) ? 5 : new Exception();
clone $a;
===expect===
PossiblyInvalidClone@3:0-3:8: cannot clone possibly non-object 5|Exception
