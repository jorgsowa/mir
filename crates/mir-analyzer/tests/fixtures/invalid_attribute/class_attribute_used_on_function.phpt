===description===
classAttributeUsedOnFunction
===file===
<?php
                    namespace Foo;

                    #[Attribute(Attribute::TARGET_CLASS)]
                    class Table {
                        public function __construct(public string $name) {}
                    }

                    #[Table("videos")]
                    function foo() : void {}
===expect===
InvalidAttribute
===ignore===
TODO
