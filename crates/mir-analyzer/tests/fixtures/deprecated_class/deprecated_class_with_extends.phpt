===description===
deprecatedClassWithExtends
===file===
<?php
                    /**
                     * @deprecated
                     */
                    class Foo { }

                    class Bar extends Foo {}
===expect===
DeprecatedClass
===ignore===
TODO
