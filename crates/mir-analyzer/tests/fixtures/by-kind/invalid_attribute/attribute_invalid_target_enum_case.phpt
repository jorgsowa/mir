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
//    ^^^^^^^^^^^ InvalidAttribute: Attribute OnlyMethods cannot be used on this target
    case Active;
}
