===description===
Functions without a @deprecated annotation are called without any diagnostic.
===file===
<?php
function greet(string $name): void {
    echo $name;
}

function test(): void {
    greet('Alice');
}
===expect===
===ignore===
TODO
