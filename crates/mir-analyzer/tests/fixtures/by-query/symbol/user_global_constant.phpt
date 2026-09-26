===description===
A user global constant resolves with its literal type.
===ignore===
===cursor===
symbol
===file===
<?php
const LIMIT = 10;
echo LIM<CURSOR>IT;
===expect===
kind: global constant LIMIT
type: 10
