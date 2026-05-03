===description===
forgetAssertionAfterReferenceModification
===file===
<?php
                    class Foo
                    {
                        public ?string $bar = null;
                    }

                    /**
                     * @psalm-assert-if-true !null $foo->bar
                     */
                    function assertBarNotNull(Foo $foo): bool
                    {
                        return $foo->bar !== null;
                    }

                    $foo = new Foo();
                    $barRef = &$foo->bar;

                    if (assertBarNotNull($foo)) {
                        $barRef = null;
                        requiresString($foo->bar);
                    }

                    function requiresString(string $_str): void {}
                
===expect===
NullArgument
===ignore===
TODO
