===description===
Same short class name in two braced namespaces: each method sees its own parameters.
===config===
suppress=UnusedFunction,UnusedMethod
===file===
<?php
namespace A {
    class K {
        public function g(int $first): void {
            /** @mir-check $first is int */
            echo $first;
        }
    }
}

namespace B {
    class K {
        public function g(string $second): void {
            /** @mir-check $second is string */
            echo $second;
        }
    }
}

namespace C {
    class K {
        public function g(bool $third): void {
            /** @mir-check $third is bool */
            echo $third;
        }
    }
}
===expect===
