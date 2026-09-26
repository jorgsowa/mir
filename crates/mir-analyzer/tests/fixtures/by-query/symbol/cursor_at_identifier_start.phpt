===description===
A cursor on the first byte of an identifier resolves it.
===cursor===
symbol
===file===
<?php
echo <CURSOR>strlen('abc');
===expect===
kind: function call strlen
type: 3
