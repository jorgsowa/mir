===description===
regular function reported
===file===
<?php
function greet(string $name): string {
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
    return 'hello';
}
