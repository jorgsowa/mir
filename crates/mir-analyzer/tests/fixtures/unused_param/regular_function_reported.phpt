===description===
regular function reported
===file===
<?php
function greet(string $name): string {
    return 'hello';
}
===expect===
UnusedParam@2:15: Parameter $name is never used
===ignore===
TODO
