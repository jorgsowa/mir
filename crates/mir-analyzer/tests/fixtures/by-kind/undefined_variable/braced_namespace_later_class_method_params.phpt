===description===
A class method in a later braced namespace block registers its parameters.
===config===
suppress=UnusedFunction,UnusedMethod
===file===
<?php
namespace A {
    function f(): void {}
}

namespace B {
    class K {
        public function g(string $m): void {
            /** @mir-check $m is string */
            echo $m;
        }
    }
}
===expect===
