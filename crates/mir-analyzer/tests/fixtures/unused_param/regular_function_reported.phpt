===source===
<?php
function greet(string $name): string {
    return 'hello';
}
===expect===
UnusedParam: $name
