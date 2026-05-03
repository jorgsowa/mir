===description===
useClosureDocblockType
===file===
<?php
                    class A {}
                    class B extends A {}

                    function takesA(A $_a) : void {}
                    function takesB(B $_b) : void {}

                    $getAButReallyB = /** @return A */ function() {
                        return new B;
                    };

                    takesA($getAButReallyB());
                    takesB($getAButReallyB());
===expect===
ArgumentTypeCoercion
===ignore===
TODO
