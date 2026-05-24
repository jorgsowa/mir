===description===
deprecatedEnumCaseFetch
===file===
<?php
enum Foo {
    case A;

    /** @deprecated */
    case B;
}

Foo::B;

===expect===
DeprecatedConstant
===ignore===
TODO
