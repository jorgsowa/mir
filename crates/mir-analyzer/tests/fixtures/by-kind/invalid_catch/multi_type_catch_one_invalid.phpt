===description===
InvalidCatch fires only for the invalid type in a multi-type (union) catch clause, leaving the valid type unflagged.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class ValidExc extends \Exception {}
class NonThrowable {}

try {
    throw new ValidExc();
} catch (ValidExc|NonThrowable $e) {}
//                ^^^^^^^^^^^^ InvalidCatch: Caught type 'NonThrowable' does not extend Throwable
===expect===
