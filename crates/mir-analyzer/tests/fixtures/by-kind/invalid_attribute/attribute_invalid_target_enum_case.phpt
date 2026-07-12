===description===
An attribute restricted to TARGET_METHOD cannot be used on an enum case.
===file===
<?php
namespace Foo;

#[Attribute(Attribute::TARGET_METHOD)]
class OnlyMethods {
}

enum Status {
    #[OnlyMethods]
    case Active;
}
===expect===
InvalidAttribute@9:6-9:17: Attribute OnlyMethods cannot be used on this target
