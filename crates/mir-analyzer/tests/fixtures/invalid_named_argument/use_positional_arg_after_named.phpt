===description===
usePositionalArgAfterNamed
===file===
<?php
                    final class Person
                    {
                        public function __construct(
                            public string $name,
                            public int $age,
                        ) { }
                    }

                    new Person(name: "", 0);
===expect===
InvalidNamedArgument
===ignore===
TODO
