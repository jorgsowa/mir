===description===
Valid cast from string to int - string is implicitly converted to int, should not emit InvalidCast

===config===
suppress=UnusedVariable
===file===
<?php
$x = (int)"42";

===expect===
