===description===
Hover on a function call shows its return type, docstring and declaration.
===cursor===
hover
===file===
<?php
/** Greets the world. */
function greet(): string { return 'hi'; }
echo gr<CURSOR>eet();
===expect===
type: string
docstring: Greets the world.
definition: test.php@3:0-3:41
