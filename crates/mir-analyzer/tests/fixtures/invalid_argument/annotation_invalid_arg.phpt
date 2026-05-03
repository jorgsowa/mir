===description===
annotationInvalidArg
===file===
<?php
                    class ParentClass {
                        public function __call(string $name, array $args) {}
                    }

                    /**
                     * @method setString(int $integer)
                     */
                    class Child extends ParentClass {}

                    $child = new Child();

                    $child->setString("five");
===expect===
InvalidArgument
===ignore===
TODO
