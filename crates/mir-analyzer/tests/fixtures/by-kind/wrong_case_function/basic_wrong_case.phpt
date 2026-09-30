===description===
Calling a function with wrong casing is reported.
===file===
<?php
function myFunc(): void {}
MYFUNC();
//<^^^^^^ WrongCaseFunction: Function name 'MYFUNC' has incorrect casing; use 'myFunc'
===expect===
