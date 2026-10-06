===description===
Method with ampersand and space
===file===
<?php
/**
 * @method void alloc(string & $result)
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @method parameter `string & $result` uses by-reference (`&`) which is not supported in @method annotations
 */
class Foo {}
===expect===
