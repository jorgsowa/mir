===description===
A `define()`d constant resolves with the literal type of its value.
===cursor===
symbol
===file===
<?php
define('MAXV', 5);
echo MA<CURSOR>XV;
===expect===
kind: global constant MAXV
type: 5
