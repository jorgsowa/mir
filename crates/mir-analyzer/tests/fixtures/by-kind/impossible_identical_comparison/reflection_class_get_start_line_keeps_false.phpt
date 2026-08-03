===description===
M25: ReflectionClass::getStartLine()'s stub docblock said @return int
(dropping the native int|false hint's |false), so `assert($line !== false)`
after it flagged ImpossibleIdenticalComparison. Fixed by matching the
sibling getFileName()'s already-correct T|false docblock.
===file===
<?php
class Foo {}
$line = (new ReflectionClass(Foo::class))->getStartLine();
assert($line !== false);
===expect===
