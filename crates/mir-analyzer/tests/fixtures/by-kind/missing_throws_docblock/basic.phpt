===description===
function throws without @throws (checked exception)
===file===
<?php
function riskyOperation(): void {
    throw new \Exception('fail');
}
===expect===
MissingThrowsDocblock@3:5-3:34: Exception Exception is thrown but not declared in @throws
