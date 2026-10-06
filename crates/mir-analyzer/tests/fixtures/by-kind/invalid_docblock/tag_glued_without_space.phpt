===description===
A tag glued to the previous one with no separating space is reported as malformed
===file===
<?php
/**
 * @template T@extends Foo
 */
class Bar {}
===expect===
InvalidDocblock@3:3-3:26: Invalid docblock: @template has a malformed type `T@extends` — a neighboring tag may be missing a space
