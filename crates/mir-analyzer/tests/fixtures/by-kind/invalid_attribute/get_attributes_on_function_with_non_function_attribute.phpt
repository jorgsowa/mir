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
UnusedPsalmSuppress@8:0-8:0: Suppress annotation for 'InvalidArgument' is never used
