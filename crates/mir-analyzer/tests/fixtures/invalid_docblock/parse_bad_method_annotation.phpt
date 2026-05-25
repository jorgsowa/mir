===description===
Parse bad method annotation
===file===
<?php
                    /**
                     * @method aaa
                     */
                    class AAA {
                        function __call() {
                            echo $b."
";
                        }
                    }
===expect===
InvalidDocblock
===ignore===
TODO
