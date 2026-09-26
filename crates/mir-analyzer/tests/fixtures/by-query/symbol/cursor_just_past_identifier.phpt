===description===
A cursor right after an identifier (before `(`) still resolves the call.
===ignore===
===cursor===
symbol
===file===
<?php
echo strlen<CURSOR>('abc');
===expect===
kind: function call strlen
type: 3
