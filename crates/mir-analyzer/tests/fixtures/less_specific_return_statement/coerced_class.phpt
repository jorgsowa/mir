===description===
coercedClass
===file===
<?php
                    class NullableClass {
                    }

                    class NullableBug {
                        /**
                         * @param class-string|null $className
                         * @return object|null
                         */
                        public static function mock($className) {
                            if (!$className) { return null; }
                            return new $className();
                        }

                        /**
                         * @return ?NullableClass
                         */
                        public function returns_nullable_class() {
                            /** @psalm-suppress ArgumentTypeCoercion */
                            return self::mock("NullableClass");
                        }
                    }
===expect===
LessSpecificReturnStatement
===ignore===
TODO
