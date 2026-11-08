===description===
`Phar::setStub()`'s `$length` is optional.
===file===
<?php
function f(Phar $phar): void { $phar->setStub('<?php __HALT_COMPILER();'); }
