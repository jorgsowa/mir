===description===
Method with ampersand and space
===file===
<?php
/**
 * @method void alloc(string & $result)
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: @method parameter `string & $result` uses by-reference (`&`) which is not supported in @method annotations
 */
class Foo {}
