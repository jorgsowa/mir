===description===
InvalidClone fires when cloning a null literal.
===config===
suppress=UnusedVariable
===file===
<?php
$x = null;
clone $x;
===expect===
InvalidClone@3:0-3:8: cannot clone non-object null
