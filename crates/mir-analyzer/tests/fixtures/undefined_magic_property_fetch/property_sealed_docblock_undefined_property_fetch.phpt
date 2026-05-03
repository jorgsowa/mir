===description===
propertySealedDocblockUndefinedPropertyFetch
===file===
<?php
                    /**
                     * @property string $foo
                     * @psalm-seal-properties
                     */
                    class A {
                        public function __get(string $name): ?string {
                            if ($name === "foo") {
                                return "hello";
                            }

                            return null;
                        }

                        /** @param mixed $value */
                        public function __set(string $name, $value): void {
                        }
                    }

                    $a = new A();
                    echo $a->bar;
===expect===
UndefinedMagicPropertyFetch
===ignore===
TODO
