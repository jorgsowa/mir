===description===
InvalidArrayAssignment does NOT fire when assigning to an actual array.
===file===
<?php
$a = [];
$a[0] = 5;
===expect===
