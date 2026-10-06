===description===
function throws without @throws (checked exception)
===file===
<?php
function riskyOperation(): void {
    throw new \Exception('fail');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Exception is thrown but not declared in @throws
}
