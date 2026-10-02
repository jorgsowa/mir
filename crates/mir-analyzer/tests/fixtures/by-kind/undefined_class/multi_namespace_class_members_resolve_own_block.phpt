===description===
Class, interface, trait and enum bodies in a later namespace block resolve against that block.
===file===
<?php
namespace A {
    class Base {}
}
namespace B {
    class Own {}
    interface Contract { public function make(): Own; }
    trait Maker { public function build(): Own { return new Own(); } }
    enum Kind { case One; public function own(): Own { return new Own(); } }
    class K extends Own implements Contract {
        use Maker;
        public function make(): Own { return new Own(); }
        public function bad(): void { new Base(); }
//                                        ^^^^ UndefinedClass: Class B\Base does not exist
    }
}
===expect===
