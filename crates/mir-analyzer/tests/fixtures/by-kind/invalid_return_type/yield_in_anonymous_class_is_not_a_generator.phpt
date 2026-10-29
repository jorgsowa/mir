===description===
A `yield` in an anonymous class's method makes that method a generator, not the enclosing function or closure.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function outer(): int {
    $o = new class {
        public function g(): \Generator {
            yield 1;
        }
    };
    return;
//  ^^^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'int'
}

$closure = function (): int {
    $o = new class {
        public function g(): \Generator {
            yield 1;
        }
    };
    return;
//  ^^^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'int'
};

function argument(): \Generator {
    $o = new class (yield 1) {
        public function __construct(mixed $v) {}
    };
    return;
}
