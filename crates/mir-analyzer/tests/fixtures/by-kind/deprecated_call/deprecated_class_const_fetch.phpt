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
DeprecatedConstant@8:6-8:7: Constant Foo::B is deprecated
