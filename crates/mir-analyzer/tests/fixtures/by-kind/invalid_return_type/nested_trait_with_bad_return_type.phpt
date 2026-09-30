===description===
Nested trait with bad return type
===file===
<?php
trait A {
    public function foo() : string {
        return 5;
//      ^^^^^^^^^ InvalidReturnType: Return type '5' is not compatible with declared 'string'
    }
}

trait B {
    use A;
}

class C {
    use B;
}
===expect===
