===description===
MagicMethodParamTypesCheckedForInterfaces
===file===
<?php
                    interface A
                    {
                        public function a(string $className): int;
                    }

                    /**
                     * @method int a(int $a)
                     */
                    interface B extends A {}
                    
===expect===
ImplementedParamTypeMismatch
===ignore===
TODO
