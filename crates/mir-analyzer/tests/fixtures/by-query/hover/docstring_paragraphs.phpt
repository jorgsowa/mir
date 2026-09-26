===description===
Hover keeps the paragraph break of a multi-paragraph docblock description.
===ignore===
===cursor===
hover
===file===
<?php
/**
 * Greets the world.
 *
 * Second paragraph.
 */
function greet(): string { return 'hi'; }
echo gr<CURSOR>eet();
===expect===
type: string
docstring: Greets the world.
docstring:
docstring: Second paragraph.
definition: test.php@7:0-7:41
