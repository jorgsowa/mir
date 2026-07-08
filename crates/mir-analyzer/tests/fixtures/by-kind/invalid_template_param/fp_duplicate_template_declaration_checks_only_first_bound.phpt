===description===
A template name declared twice with different bounds (malformed docblock)
only checks the first declaration's bound — checking the single resolved
binding against every duplicate's own bound treated each extra
declaration as an unrelated, additional constraint the caller never
agreed to satisfy
===config===
suppress=UnusedParam
===file===
<?php
declare(strict_types=1);
class Foo {}
class Bar {}
class Sub extends Foo {}

/**
 * @template T of Foo
 * @template T of Bar
 * @param T $x
 */
function process($x): void {}

process(new Sub());
===expect===
