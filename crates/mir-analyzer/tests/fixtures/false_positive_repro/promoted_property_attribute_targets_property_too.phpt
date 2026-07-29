===description===
A promoted constructor parameter is reflectable via both `ReflectionParameter`
(target=parameter) and `ReflectionProperty` (target=property) — verified
against real PHP semantics. An attribute restricted to TARGET_PROPERTY alone
(no TARGET_PARAMETER) must still be accepted on a promoted param, since it
also declares a property. Covers class and trait constructors.
===config===
suppress=UnusedParam
===file===
<?php
#[Attribute(Attribute::TARGET_PROPERTY)]
class OnlyProperty {}

class Foo {
    public function __construct(
        #[OnlyProperty] public int $id,
    ) {}
}

trait HasLogger {
    public function __construct(
        #[OnlyProperty] private readonly string $name,
    ) {}
}
===expect===
