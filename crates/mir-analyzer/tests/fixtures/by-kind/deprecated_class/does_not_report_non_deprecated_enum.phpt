===description===
Sibling of deprecated_enum_as_param: a plain enum stays silent.
===config===
suppress=UnusedParam
===file===
<?php
enum Status { case A; case B; }

function foo(Status $s): void {}
===expect===
