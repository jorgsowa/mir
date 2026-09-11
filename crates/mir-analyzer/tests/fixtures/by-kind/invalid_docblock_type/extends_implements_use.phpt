===description===
Generic-inheritance tags are checked too: a backslash-qualified keyword is
invalid, while backslash-qualified real classes are valid fully qualified
names.
===file===
<?php
/**
 * @extends \ArrayObject<int, string>
 * @implements \Traversable
 * @use \mixed
 */
class Bag {}
===expect===
InvalidDocblockType@5:8-5:14: Invalid docblock type: @use backslash-qualified non-class type '\mixed' is not a fully qualified name
