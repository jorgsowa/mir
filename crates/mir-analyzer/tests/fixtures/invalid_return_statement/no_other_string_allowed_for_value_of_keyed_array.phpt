===description===
noOtherStringAllowedForValueOfKeyedArray
===file===
<?php
                    /**
                     * @return value-of<array{a: "foo", b: "bar"}>
                     */
                    function getValue() {
                        return "adams";
                    }
                
===expect===
InvalidReturnStatement
===ignore===
TODO
