===description===
P2: A function/method declared `: never` that can fall off the end must be
flagged — never means the function must always throw, exit, or otherwise diverge.
A properly-diverging `: never` body must NOT be flagged.
===file===
<?php

function falls_through(): never {
//                        ^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'never'
}

function also_falls_through(): never {
//                             ^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'never'
    echo "doing work";
}

function properly_diverges(): never {
    throw new RuntimeException("always throws");
}

class Foo {
    public function method_falls_through(): never {
//                                          ^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'never'
    }

    public function method_diverges(): never {
        exit(1);
    }
}
===expect===
