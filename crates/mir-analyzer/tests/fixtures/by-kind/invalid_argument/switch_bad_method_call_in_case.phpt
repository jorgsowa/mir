===description===
Switch bad method call in case
===config===
suppress=UnusedParam
===file===
<?php
function f(string $p): void { }

switch (true) {
    case $q = (bool) rand(0,1):
        f($q); // this type problem is not detected
        break;
}
===expect===
InvalidArgument@6:10-6:12: Argument $p of f() expects 'string', got 'bool'
