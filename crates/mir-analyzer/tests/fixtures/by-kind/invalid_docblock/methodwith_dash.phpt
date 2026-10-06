===description===
Methodwith dash
===file===
<?php
/**
 * A test class
 *
 * @method ClientInterface exchange-connect(array $options = [])
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @method has invalid method name `exchange-connect`: must be a valid PHP identifier
 */
abstract class TestClassA {}
===expect===
