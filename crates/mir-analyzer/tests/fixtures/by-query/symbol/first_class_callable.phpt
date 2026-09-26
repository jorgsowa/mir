===description===
A first-class callable resolves to the function, typed as the Closure the expression produces.
===ignore===
===cursor===
symbol
===file===
<?php
$f = str<CURSOR>len(...);
===expect===
kind: function call strlen
type: Closure(string): int<0, max>
