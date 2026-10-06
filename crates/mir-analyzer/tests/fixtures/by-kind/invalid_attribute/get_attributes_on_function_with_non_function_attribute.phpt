===description===
Get attributes on function with non function attribute
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class Attr {}

function foo(): void {}

/** @suppress InvalidArgument */
//            ^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'InvalidArgument' is never used
$r = new ReflectionFunction("foo");
$r->getAttributes(Attr::class);
