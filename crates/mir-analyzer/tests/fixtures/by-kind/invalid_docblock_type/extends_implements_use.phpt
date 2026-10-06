===description===
Inheritance tags reject backslash-qualified keywords but allow class names.
===file===
<?php
/**
 * @extends \ArrayObject<int, string>
 * @implements \Traversable
 * @use \mixed
//      ^^^^^^ InvalidDocblockType: Invalid docblock type: @use backslash-qualified non-class type '\mixed' is not a fully qualified name
 */
class Bag {}
