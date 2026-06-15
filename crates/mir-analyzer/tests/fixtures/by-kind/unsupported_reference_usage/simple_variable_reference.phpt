===description===
UnsupportedReferenceUsage fires for a simple reference assignment ($b = &$a).
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
$b = &$a;

===expect===
UnsupportedReferenceUsage@3:0-3:8: Reference assignment is not supported
