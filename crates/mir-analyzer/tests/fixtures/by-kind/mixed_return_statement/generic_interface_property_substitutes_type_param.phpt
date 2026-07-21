===description===
`@property T $value` on a generic interface substitutes the receiver's own
concrete type argument, instead of leaking the raw unbound template.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam
===file===
<?php
/**
 * @template T
 * @property T $value
 */
interface Box {}

/** @param Box<int> $b */
function test(Box $b): void {
    /** @mir-check $b->value is int */
    $_ = 1;
}
===expect===
