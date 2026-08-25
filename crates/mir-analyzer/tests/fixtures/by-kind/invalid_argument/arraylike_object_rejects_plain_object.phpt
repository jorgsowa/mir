===description===
`arraylike-object<K, V>` must not degrade to accepting any object: a plain
class that implements none of ArrayAccess/Countable/Traversable is still
rejected.
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
