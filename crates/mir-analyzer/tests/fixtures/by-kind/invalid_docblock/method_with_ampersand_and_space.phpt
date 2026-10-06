===description===
Method with ampersand and space
===file===
<?php
/**
 * @method void alloc(string & $result)
 */
class Foo {}
===expect===
InvalidDocblock@3:3-3:39: Invalid docblock: @method parameter `string & $result` uses by-reference (`&`) which is not supported in @method annotations
