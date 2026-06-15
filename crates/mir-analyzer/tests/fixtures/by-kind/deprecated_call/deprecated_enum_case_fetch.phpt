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
DeprecatedConstant@9:5-9:6: Constant Foo::B is deprecated
