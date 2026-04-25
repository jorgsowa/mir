===file===
<?php
function pair(int $a, int $b): void {}
pair(a: 1, 2);
===expect===
ParseError: Parse error: cannot use positional argument after named argument
