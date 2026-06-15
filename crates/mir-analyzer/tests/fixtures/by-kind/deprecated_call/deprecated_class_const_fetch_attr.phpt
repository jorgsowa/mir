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

===expect===
DeprecatedConstant@9:5-9:6: Constant Foo::B is deprecated
