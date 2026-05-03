===description===
deprecatedEnumCaseFetchAttr
===file===
<?php
                    enum Foo {
                        case A;

                        #[Deprecated]
                        case B;
                    }

                    Foo::B;
                
===expect===
DeprecatedConstant
===ignore===
TODO
