===description===
function throws without @throws (checked exception)
===file===
<?php
function riskyOperation(): void {
    throw new \Exception('fail');
}
===expect===
MissingThrowsDocblock@3:4-3:33: Exception Exception is thrown but not declared in @throws
