===description===
FP-A: extension_loaded() guard works for well-known PHP extensions whose classes
exist in stubs. Both the guard form and the post-guard (early-exit) form must
not emit UndefinedClass even without stubs (the guard is sufficient).
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

// intl extension — IntlChar is in stubs
if (extension_loaded('intl')) {
    $char = \IntlChar::chr(65);
}

// redis extension — Redis is in stubs
if (extension_loaded('redis')) {
    $redis = new \Redis();
}

// Negated early-exit form for a stub-less custom extension
function connectToCustomExt(): void {
    if (!extension_loaded('my_db_ext')) {
        throw new \RuntimeException('my_db_ext required');
    }
    $conn = new \MyDbExt\Connection();
    $conn->open();
}
===expect===
