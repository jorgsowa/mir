===description===
Calling a built-in function with wrong casing is reported.
===file===
<?php
STRLEN("hello");
===expect===
WrongCaseFunction@2:1-2:7: Function name 'STRLEN' has incorrect casing; use 'strlen'
