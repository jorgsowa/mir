===description===
badAssignmentAsWell
===file===
<?php
$a = "hello";
$a->foo = "bar";
===expect===
InvalidPropertyAssignment
===ignore===
TODO
