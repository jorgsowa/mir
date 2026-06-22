===description===
A class name containing non-ASCII characters is still subject to ASCII case
checks: only the ASCII letters must match the declaration's casing.
===config===
suppress=UnusedVariable
===file===
<?php
class GrüBar {}
$x = new grübar();
===expect===
WrongCaseClass@3:9-3:15: Class name 'grübar' has incorrect casing; use 'GrüBar'
