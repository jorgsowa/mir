===description===
A tag glued to the previous one with no separating space is reported as malformed
===file===
<?php
/**
 * @template T@extends Foo
// ^^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @template has a malformed type `T@extends` — a neighboring tag may be missing a space
 */
class Bar {}
