===description===
magicGetterUndefinedProperty
===file===
<?php
                    /**
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

                        public function badGet(): void {
                            $this->__get("foo");
                        }
                    }
===expect===
UndefinedThisPropertyFetch
===ignore===
TODO
