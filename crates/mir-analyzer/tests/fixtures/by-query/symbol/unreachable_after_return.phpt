===description===
Code after a `return` still resolves, so navigation works in dead code.
===ignore===
===cursor===
symbol
===file===
<?php
final class Greeter {}
function f(): void {
    return;
    $g = new Gree<CURSOR>ter();
}
===expect===
kind: class Greeter
type: Greeter
