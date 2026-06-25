===description===
InvalidCatch fires only for the invalid type in a multi-type (union) catch clause, leaving the valid type unflagged.
===config===
suppress=UnusedVariable,MissingThrowsDocblock
===file===
<?php
class ValidExc extends \Exception {}
class NonThrowable {}

try {
    throw new ValidExc();
} catch (ValidExc|NonThrowable $e) {}
===expect===
InvalidCatch@7:18-7:30: Caught type 'NonThrowable' does not extend Throwable
