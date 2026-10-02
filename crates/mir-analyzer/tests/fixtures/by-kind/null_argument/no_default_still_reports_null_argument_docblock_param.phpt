===description===
Without a default, a docblock non-null param still rejects null.
===config===
suppress=UnusedParam
===file===
<?php
/** @param string $f */
function plain($f): void {}
plain(null);
//    ^^^^ NullArgument: Argument $f of plain() cannot be null
===expect===
