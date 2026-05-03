===description===
missingAttributeOnParam
===file===
<?php
                    use FooBarPure;

                    function foo(#[Pure] string $str) : void {}
===expect===
UndefinedAttributeClass
===ignore===
TODO
