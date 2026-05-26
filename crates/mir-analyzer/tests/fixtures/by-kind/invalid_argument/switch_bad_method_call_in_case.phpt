===description===
Switch bad method call in case
===file===
<?php
function f(string $p): void { }

switch (true) {
    case $q = (bool) rand(0,1):
        f($q); // this type problem is not detected
        break;
}
===expect===
InvalidScalarArgument
===ignore===
TODO
