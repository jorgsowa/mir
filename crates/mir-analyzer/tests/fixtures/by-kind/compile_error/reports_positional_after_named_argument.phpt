===description===
reports positional after named argument
===file===
<?php
function pair(int $a, int $b): int { return $a + $b; }
pair(a: 1, 2);
//         ^ CompileError: Compile error: Cannot use positional argument after named argument [positional-after-named-argument]
