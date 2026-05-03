===description===
doubleForeachWithInnerUnusedValue
===file===
<?php
                    /**
                     * @param non-empty-list<list<int>> $arr
                     * @return list<int>
                     */
                    function f(array $arr): array {
                        foreach ($arr as $elt) {
                            foreach ($elt as $subelt) {}
                        }
                        return $elt;
                    }
                
===expect===
UnusedForeachValue
===ignore===
TODO
