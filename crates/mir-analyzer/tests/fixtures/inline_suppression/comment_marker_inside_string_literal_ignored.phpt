===description===
`//`/`#` inside a same-line string literal was mistaken for a real trailing
comment — a plain string statement containing `// @psalm-suppress …` text
wrongly registered a same-line suppression, silencing a genuinely
unrelated issue on the very same physical line.
===config===
suppress=UnusedVariable
===file===
<?php
$x = "// @psalm-suppress UndefinedClass"; new NoSuchClass();
===expect===
UndefinedClass@2:46-2:57: Class NoSuchClass does not exist
