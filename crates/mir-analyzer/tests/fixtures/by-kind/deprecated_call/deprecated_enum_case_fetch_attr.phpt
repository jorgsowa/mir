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
//   ^ DeprecatedConstant: Constant Foo::B is deprecated

===expect===
