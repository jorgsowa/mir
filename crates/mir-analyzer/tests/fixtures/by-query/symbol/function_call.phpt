===description===
A function call resolves to the function and its return type.
===cursor===
symbol
===file===
<?php
function greet(): string { return 'hi'; }
echo gr<CURSOR>eet();
===expect===
kind: function call greet
type: string
