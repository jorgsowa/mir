===description===
invalidArgumentCallableWithoutArgsUnion
===file===
<?php
                    function foo(int $a): void {}

                    /**
                     * @param callable()|float $callable
                     * @return void
                     */
                    function acme($callable) {}
                    acme("foo");
===expect===
UnusedParam@2:33: Parameter $a is never used
UnusedParam@8:34: Parameter $callable is never used
InvalidArgument@9:25: Argument $callback of typed_callable() expects 'callable with 0 required parameter(s)', got 'callable with 1 required parameter(s)'
