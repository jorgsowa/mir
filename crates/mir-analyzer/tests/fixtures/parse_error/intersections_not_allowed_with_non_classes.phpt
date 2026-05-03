===description===
intersectionsNotAllowedWithNonClasses
===file===
<?php
                    interface A {
                    }
                    function foo (A&string $test): A&string {
                        return $test;
                    }
===expect===
ParseError
===ignore===
TODO
