===description===
A class named in a parameter type hint resolves to the class.
===cursor===
symbol
===file===
<?php
final class Greeter {}
function f(Gree<CURSOR>ter $g): void {}
===expect===
kind: class Greeter
type: class-string
