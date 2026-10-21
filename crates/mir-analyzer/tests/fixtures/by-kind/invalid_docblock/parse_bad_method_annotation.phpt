===description===
Parse bad method annotation
===file===
<?php
                    /**
                     * @method aaa
                     */
                    class AAA {
                        function __call($name, $args) {
                            echo $b."
//                               ^^ UndefinedVariable: Variable $b is not defined
";
                        }
                    }
