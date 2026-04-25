===file===
<?php
function greet(string $name): void {
    echo $name;
}

function test(): void {
    greet('Alice');
}
===expect===
