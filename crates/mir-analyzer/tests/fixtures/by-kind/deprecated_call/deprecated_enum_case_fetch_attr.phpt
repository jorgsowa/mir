===description===
Deprecated enum case fetch attr
===file===
<?php
enum Foo {
    case A;

    #[Deprecated]
    case B;
}

Foo::B;

===expect===
DeprecatedConstant@9:6-9:7: Constant Foo::B is deprecated
