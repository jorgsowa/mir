===description===
undefinedMixinClassWithMethodCall_WithMagicMethod
===file===
<?php
                    /**
                     * @method baz()
                     * @mixin B
                     */
                    class A {
                        public function __call(string $name, array $arguments) {}
                    }

                    (new A)->foo();
===expect===
UndefinedMagicMethod
===ignore===
TODO
