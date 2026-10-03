===description===
A `#`/`//`-looking line INSIDE a heredoc body (e.g. embedded shell/SQL)
was indistinguishable from a real suppression comment -- the scanner had
no cross-line heredoc-body tracking at all, so this genuinely bogus
`@mir-ignore-file` embedded in a shell script silently suppressed
UndefinedClass for the whole file.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$script = <<<BASH
#!/bin/bash
# @mir-ignore-file UndefinedClass
echo hello
BASH;
new NoSuchClass();
//  ^^^^^^^^^^^ UndefinedClass: Class NoSuchClass does not exist
===expect===
