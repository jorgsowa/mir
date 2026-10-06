===description===
An unclosed generic in @extends is reported instead of silently swallowed
===file===
<?php
/** @template T */
class Base {}

/**
 * @template T
 * @extends Base<T
// ^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @extends has unclosed generic type `Base<T`
 */
class Derived extends Base {}
===expect===
