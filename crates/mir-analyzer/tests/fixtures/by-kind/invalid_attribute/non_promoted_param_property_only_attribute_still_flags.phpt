===description===
A plain (non-promoted) parameter is only reflectable as a `ReflectionParameter`
— an attribute restricted to TARGET_PROPERTY alone must still be rejected on
it, unlike on a promoted param.
===config===
suppress=UnusedParam
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class OnlyProperty {}

function foo(#[OnlyProperty] int $id): void {}
===expect===
InvalidAttribute@5:15-5:27: Attribute OnlyProperty cannot be used on this target
