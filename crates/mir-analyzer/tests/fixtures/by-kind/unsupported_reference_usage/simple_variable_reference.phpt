===description===
UnsupportedReferenceUsage fires for a simple reference assignment ($b = &$a).
===file===
<?php
$a = "hello";
$b = &$a;

===expect===
UnsupportedReferenceUsage@3:1-3:9: Reference assignment is not supported
