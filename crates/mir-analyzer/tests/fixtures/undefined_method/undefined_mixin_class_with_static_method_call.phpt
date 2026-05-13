===description===
undefinedMixinClassWithStaticMethodCall
===file===
<?php
                    /** @mixin B */
                    class A {}

                    A::foo();
===expect===
UndefinedMethod@5:20: Method A::foo() does not exist
