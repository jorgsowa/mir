===description===
Simple reference assignment ($b = &$a) does not fire UnsupportedReferenceUsage.
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
$b = &$a;

===expect===
