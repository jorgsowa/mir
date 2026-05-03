===description===
undefinedMixinClassWithStaticMethodCall
===file===
<?php
                    /** @mixin B */
                    class A {}

                    A::foo();
===expect===
UndefinedMethod
===ignore===
TODO
