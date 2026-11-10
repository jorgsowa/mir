===description===
closure declared in top-level code still needs @throws
===file===
<?php
$fail = function (): void {
    throw new \Exception('fail');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Exception is thrown but not declared in @throws
};
$fail();
