===description===
regular function reported
===file===
<?php
function greet(string $name): string {
    return 'hello';
}
===expect===
UnusedParam@2:16: Parameter $name is never used
