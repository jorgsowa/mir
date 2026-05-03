===description===
getClassExteriorArgStringType
===file===
<?php
                    /** @return void */
                    function foo(Exception $e) {
                        switch (get_class($e)) {
                            case "InvalidArgumentException":
                                break;
                        }
                    }
===expect===
TypeDoesNotContainType
===ignore===
TODO
