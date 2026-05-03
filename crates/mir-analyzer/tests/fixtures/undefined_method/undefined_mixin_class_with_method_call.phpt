===description===
undefinedMixinClassWithMethodCall
===file===
<?php
                    /** @mixin B */
                    class A {}

                    (new A)->foo();
===expect===
UndefinedMethod
===ignore===
TODO
