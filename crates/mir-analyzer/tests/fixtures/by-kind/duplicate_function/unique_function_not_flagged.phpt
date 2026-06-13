===description===
DuplicateFunction does NOT fire when a function is declared only once.
===file===
<?php
function greet(string $name): string {
    return "Hello, $name!";
}

echo greet('World');
===expect===
