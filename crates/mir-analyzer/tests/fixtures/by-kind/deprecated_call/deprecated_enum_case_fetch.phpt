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
//   ^ DeprecatedConstant: Constant Foo::B is deprecated

===expect===
