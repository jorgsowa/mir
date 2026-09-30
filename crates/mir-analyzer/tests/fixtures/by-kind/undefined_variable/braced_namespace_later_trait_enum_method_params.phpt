===description===
Trait and enum methods in a later braced namespace block register their parameters.
===config===
suppress=UnusedFunction,UnusedMethod
===file===
<?php
namespace A {
    function f(): void {}
}

namespace B {
    trait T {
        public function g(int $n): void {
            /** @mir-check $n is int */
            echo $n;
        }
    }

    enum E {
        case One;

        public function h(string $s): void {
            /** @mir-check $s is string */
            echo $s;
        }
    }
}
===expect===
