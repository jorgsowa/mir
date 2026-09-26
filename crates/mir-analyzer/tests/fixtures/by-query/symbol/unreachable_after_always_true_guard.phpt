===description===
Code after an always-true early return still resolves, so navigation works in code mir proves unreachable.
===ignore===
===cursor===
symbol
===file===
<?php
final class Greeter {}
function f(Greeter $g): void {
    if ($g instanceof Greeter) { return; }
    $h = new Gree<CURSOR>ter();
}
===expect===
kind: class Greeter
type: Greeter
