===description===
An unclosed generic in @extends is reported instead of silently swallowed
===file===
<?php
/** @template T */
class Base {}

/**
 * @template T
 * @extends Base<T
 */
class Derived extends Base {}
===expect===
InvalidDocblock@7:3-7:18: Invalid docblock: @extends has unclosed generic type `Base<T`
