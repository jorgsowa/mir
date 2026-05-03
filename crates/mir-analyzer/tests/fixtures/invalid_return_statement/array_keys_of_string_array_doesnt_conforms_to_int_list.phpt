===description===
arrayKeysOfStringArrayDoesntConformsToIntList
===file===
<?php
                    /**
                     * @param array<string, mixed> $array
                     * @return list<int>
                     */
                    function getKeys(array $array) {
                        return array_keys($array);
                    }
                
===expect===
InvalidReturnStatement
===ignore===
TODO
