===description===
Nested trait with bad return type
===file===
<?php
trait A {
    public function foo() : string {
        return 5;
    }
}

trait B {
    use A;
}

class C {
    use B;
}
===expect===
InvalidReturnType@4:9-4:18: Return type '5' is not compatible with declared 'string'
