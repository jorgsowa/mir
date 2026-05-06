===description===
function throws without @throws
===file===
<?php
function riskyOperation(): void {
    throw new \RuntimeException('fail');
}
===expect===
MissingThrowsDocblock@3:4: Exception RuntimeException is thrown but not declared in @throws
