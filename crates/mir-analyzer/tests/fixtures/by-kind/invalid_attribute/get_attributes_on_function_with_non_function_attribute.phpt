===description===
Get attributes on function with non function attribute
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class Attr {}

function foo(): void {}

/** @suppress InvalidArgument */
$r = new ReflectionFunction("foo");
$r->getAttributes(Attr::class);

===expect===
UnusedSuppress@7:14-7:29: Suppress annotation for 'InvalidArgument' is never used
