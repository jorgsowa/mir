===description===
`arraylike-object` rejects objects without array-like capabilities.
===config===
suppress=UnusedParam
===file===
<?php
final class PlainObject {}

/** @param arraylike-object<string, int> $bag */
function takesArraylike($bag): void {}

takesArraylike(new PlainObject());
===expect===
InvalidArgument@7:15-7:32: Argument $bag of takesArraylike() expects 'ArrayAccess<string, int>&Countable&Traversable<string, int>', got 'PlainObject'
