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
UndefinedVariable@7:34-7:36: Variable $b is not defined
