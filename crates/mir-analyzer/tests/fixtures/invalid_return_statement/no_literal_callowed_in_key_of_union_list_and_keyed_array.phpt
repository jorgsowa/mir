===description===
noLiteralCAllowedInKeyOfUnionListAndKeyedArray
===file===
<?php
                    /**
                     * @return key-of<list<int>|array{a: int, b: int}>
                     */
                    function getKey() {
                        return "c";
                    }
                
===expect===
InvalidReturnStatement
===ignore===
TODO
