===description===
UndefinedDocblockClass does NOT fire when `@mixin`, `@property`, and
`@method` docblock tags all name classes that exist.
===file===
<?php
class Existing {}

/**
 * @mixin Existing
 * @property Existing $thing
 * @method Existing getThing(Existing $arg)
 */
class A {}
===expect===
