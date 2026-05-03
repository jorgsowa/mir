===description===
invalidGenericClone
===file===
<?php
                    /**
                     * @template T as int|string
                     * @param T $a
                     */
                    function foo($a): void {
                        clone $a;
                    }
===expect===
InvalidClone
===ignore===
TODO
