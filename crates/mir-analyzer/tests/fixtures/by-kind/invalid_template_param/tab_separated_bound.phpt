===description===
A tab-separated `@template T of Bound` line binds the bound instead of dropping it
===file===
<?php
/**
 * @template	T	of	ArrayAccess
 * @param T $x
 */
function f($x): void {}

f(5);
===expect===
UnusedParam@6:11-6:13: Parameter $x is never used
InvalidTemplateParam@8:0-8:4: Template type 'T' inferred as '5' does not satisfy bound 'ArrayAccess'
