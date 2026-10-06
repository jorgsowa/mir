===description===
Methodwith dash
===file===
<?php
/**
 * A test class
 *
 * @method ClientInterface exchange-connect(array $options = [])
 */
abstract class TestClassA {}
===expect===
InvalidDocblock@5:3-5:64: Invalid docblock: @method has invalid method name `exchange-connect`: must be a valid PHP identifier
