===description===
magicInterfaceWrongPropertyWrite
===file===
<?php
                    /**
                     * @property-write string $foo
                     * @psalm-seal-properties
                     */
                    interface GetterSetter {
                        /** @return mixed */
                        public function __get(string $key);
                        /** @param mixed $value */
                        public function __set(string $key, $value) : void;
                    }

                    /** @psalm-suppress NoInterfaceProperties */
                    function getFoo(GetterSetter $o) : void {
                        $o->bar = "hello";
                    }
===expect===
UndefinedMagicPropertyAssignment
===ignore===
TODO
