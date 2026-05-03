===description===
variadicArgumentWithNoNamedArgumentsPreventsPassingArrayWithStringKey
===file===
<?php
                    /**
                     * @no-named-arguments
                     * @psalm-return list<int>
                     */
                    function foo(int ...$values): array
                    {
                        return $values;
                    }

                    foo(...["a" => 0]);
                
===expect===
NamedArgumentNotAllowed
===ignore===
TODO
