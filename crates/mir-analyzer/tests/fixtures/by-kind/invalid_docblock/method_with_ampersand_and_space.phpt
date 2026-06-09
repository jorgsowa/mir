===description===
Method with ampersand and space
===file===
<?php
/**
 * @method void alloc(string & $result)
 */
class Foo {}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @method parameter `string & $result` uses by-reference (`&`) which is not supported in @method annotations
