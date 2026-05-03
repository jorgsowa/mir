===description===
invalidDocblockParamDefault
===file===
<?php
                    /**
                     * @param  int $p
                     * @return void
                     */
                    function f($p = false) {}
===expect===
InvalidParamDefault
===ignore===
TODO
