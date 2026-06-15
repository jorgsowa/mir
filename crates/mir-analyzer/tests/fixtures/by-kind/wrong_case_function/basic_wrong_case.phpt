===description===
Calling a function with wrong casing is reported.
===file===
<?php
function myFunc(): void {}
MYFUNC();
===expect===
WrongCaseFunction@3:0-3:6: Function name 'MYFUNC' has incorrect casing; use 'myFunc'
