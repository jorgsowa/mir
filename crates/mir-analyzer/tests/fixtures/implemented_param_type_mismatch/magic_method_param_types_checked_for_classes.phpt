===description===
MagicMethodParamTypesCheckedForClasses
===file===
<?php
                    class A
                    {
                        public function a(int $className): int { return 0; }
                    }

                    /**
                     * @method int a(string $a)
                     */
                    class B extends A {}
                    
===expect===
ImplementedParamTypeMismatch
===ignore===
TODO
