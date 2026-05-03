===description===
nativeTypeIntersectionAsClassPropertyUsingUnknownInterfaces
===file===
<?php
                    class C {
                        private ExampleUnknownA&ExampleUnknownB $other;
                        public function __construct()
                        {
                            $this->other = new ExampleUnknownAB();
                        }
                    }
                '
                // @todo decide whether a fall-back should be implemented, that allows to by-pass this failure (opt-in config)
                // `UndefinedClass - src/somefile.php:3:33 - Class, interface or enum named ExampleUnknownB does not exist`
===expect===
UndefinedClass
===ignore===
TODO
