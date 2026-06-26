===description===
InvalidClone fires when cloning a false literal (bool subtype).
===config===
suppress=UnusedVariable
===file===
<?php
$x = false;
clone $x;
===expect===
InvalidClone@3:0-3:8: cannot clone non-object false
