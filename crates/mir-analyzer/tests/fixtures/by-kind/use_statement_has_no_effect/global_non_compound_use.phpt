===description===
A single-segment `use` without an alias outside any namespace has no effect.
===file===
<?php
use Foo;
//  ^^^ UseStatementHasNoEffect: The use statement with non-compound name 'Foo' has no effect
use function strlen;
//           ^^^^^^ UseStatementHasNoEffect: The use statement with non-compound name 'strlen' has no effect
use Bar, Baz\Qux;
//  ^^^ UseStatementHasNoEffect: The use statement with non-compound name 'Bar' has no effect
