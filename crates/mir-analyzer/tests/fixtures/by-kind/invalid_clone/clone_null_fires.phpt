===description===
InvalidClone fires when cloning a null literal.
===config===
suppress=UnusedVariable
===file===
<?php
$x = null;
clone $x;
//<^^^^^^^^ InvalidClone: cannot clone non-object null
===expect===
