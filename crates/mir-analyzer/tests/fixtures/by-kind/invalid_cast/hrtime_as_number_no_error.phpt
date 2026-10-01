===description===
hrtime(true) returns int, not int|false — casting to string must not emit InvalidCast

===config===
suppress=UnusedVariable
===file===
<?php
$ns = hrtime(true);
$str = (string)$ns;

===expect===
