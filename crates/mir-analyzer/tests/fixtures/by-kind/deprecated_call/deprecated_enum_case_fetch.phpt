===description===
Deprecated enum case fetch
===file===
<?php
enum Foo {
    case A;

    /** @deprecated */
    case B;
}

Foo::B;

===expect===
DeprecatedConstant@9:6-9:7: Constant Foo::B is deprecated
