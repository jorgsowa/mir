===description===
SKIPPED-referenceToTypedArrayConstrainsAssignment
===file===
<?php
                    class Foo
                    {
                        /** @var list<int> */
                        public array $arr = [];

                        public function __construct()
                        {
                            assert(isset($this->arr[0]));
                            $int = &$this->arr[0];
                            $int = (string) $int;
                        }
                    }
                
===expect===
ReferenceConstraintViolation
===ignore===
TODO
