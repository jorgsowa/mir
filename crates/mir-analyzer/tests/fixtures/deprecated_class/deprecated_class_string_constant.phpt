===description===
deprecatedClassStringConstant
===file===
<?php
                    /**
                     * @deprecated
                     */
                    class Foo {}

                    echo Foo::class;
===expect===
DeprecatedClass
===ignore===
TODO
