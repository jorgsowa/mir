===description===
A `#`/`//`-looking line INSIDE a heredoc body (e.g. embedded shell/SQL)
was indistinguishable from a real suppression comment -- the scanner had
no cross-line heredoc-body tracking at all, so this genuinely bogus
`@mir-ignore-file` embedded in a shell script silently suppressed
UndefinedClass for the whole file.
===config===
suppress=UnusedVariable
===file===
<?php
$script = <<<BASH
#!/bin/bash
# @mir-ignore-file UndefinedClass
echo hello
BASH;
new NoSuchClass();
===expect===
UndefinedClass@7:4-7:15: Class NoSuchClass does not exist
