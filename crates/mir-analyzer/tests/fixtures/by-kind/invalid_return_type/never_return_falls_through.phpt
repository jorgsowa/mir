===description===
P2: A function/method declared `: never` that can fall off the end must be
flagged — never means the function must always throw, exit, or otherwise diverge.
A properly-diverging `: never` body must NOT be flagged.
===file===
<?php

function falls_through(): never {
}

function also_falls_through(): never {
    echo "doing work";
}

function properly_diverges(): never {
    throw new RuntimeException("always throws");
}

class Foo {
    public function method_falls_through(): never {
    }

    public function method_diverges(): never {
        exit(1);
    }
}
===expect===
InvalidReturnType@3:32-4:33: Return type 'void' is not compatible with declared 'never'
InvalidReturnType@6:37-8:38: Return type 'void' is not compatible with declared 'never'
InvalidReturnType@15:50-16:51: Return type 'void' is not compatible with declared 'never'
