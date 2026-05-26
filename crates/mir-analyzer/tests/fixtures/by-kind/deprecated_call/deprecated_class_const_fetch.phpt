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
DeprecatedConstant
===ignore===
TODO
