===description===
$argv and $argc are auto-populated by PHP in CLI scripts (register_argc_argv is on
by default for CLI). Accessing them at global scope must not emit UndefinedVariable.
===file===
<?php
$a = $argv[1];
/** @mir-check $a is mixed */
$b = $argc;
/** @mir-check $b is mixed */
===expect===
