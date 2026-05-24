===description===
reports positional after named argument
===file===
<?php
function pair(int $a, int $b): void {}
pair(a: 1, 2);
===expect===
ParseError@3:12: Parse error: cannot use positional argument after named argument
