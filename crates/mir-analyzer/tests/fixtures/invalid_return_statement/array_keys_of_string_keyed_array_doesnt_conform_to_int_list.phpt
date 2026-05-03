===description===
arrayKeysOfStringKeyedArrayDoesntConformToIntList
===file===
<?php
                    /**
                     * @return list<int>
                     */
                    function getKeys() {
                        return array_keys(["foo" => 42, "bar" => 42]);
                    }
                
===expect===
InvalidReturnStatement
===ignore===
TODO
