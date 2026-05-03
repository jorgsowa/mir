===description===
badArrayByRef
===file===
<?php
                    function fooFoo(array &$a): void {}
                    fooFoo([1, 2, 3]);
===expect===
InvalidPassByReference
===ignore===
TODO
