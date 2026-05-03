===description===
useUnpackedInvalidNamedArgument
===file===
<?php
                    class CustomerData {
                        public function __construct(
                            public string $name,
                            public string $email,
                            public int $age,
                        ) {}
                    }

                    /**
                     * @param array{aage: int, name: string, email: string} $input
                     */
                    function foo(array $input) : CustomerData {
                        return new CustomerData(...$input);
                    }
===expect===
InvalidNamedArgument
===ignore===
TODO
