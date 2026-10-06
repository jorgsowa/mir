===description===
Deprecated class const fetch attr
===file===
<?php
class Foo {
    const A = 1;

    #[Deprecated]
    const B = 2;
}

Foo::B;
//   ^ DeprecatedConstant: Constant Foo::B is deprecated
