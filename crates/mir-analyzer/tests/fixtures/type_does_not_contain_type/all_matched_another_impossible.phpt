===description===
allMatchedAnotherImpossible
===file===
<?php
                    function foo() : string {
                        $a = rand(0, 1) ? "a" : "b";
                        return match ($a) {
                            "a" => "hello",
                            "b" => "goodbye",
                            "c" => "impossible",
                        };
                    }
===expect===
TypeDoesNotContainType
===ignore===
TODO
