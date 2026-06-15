===description===
Deprecated class const fetch
===file===
<?php
class Foo {
    const A = 1;

    /** @deprecated */
    const B = 2;
}
Foo::B;

===expect===
DeprecatedConstant@8:5-8:6: Constant Foo::B is deprecated
