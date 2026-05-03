===description===
regular function reported
===file===
<?php
function greet(string $name): string {
    return 'hello';
}
===expect===
UnusedParam: Parameter $name is never used
===ignore===
TODO
