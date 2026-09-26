===description===
`Foo::class` resolves the class token to the class.
===cursor===
symbol
===file===
<?php
final class Greeter {}
$c = Gree<CURSOR>ter::class;
===expect===
kind: class Greeter
type: class-string
