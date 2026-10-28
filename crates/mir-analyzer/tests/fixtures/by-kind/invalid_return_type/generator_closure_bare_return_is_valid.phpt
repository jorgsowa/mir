===description===
A bare `return;` ends a generator closure the same way it ends a generator function
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$early = function (): \Generator {
    if (rand(0, 1)) {
        return;
    }
    yield 1;
};

$iterable = static function (): iterable {
    yield 1;
    return;
};

$notAGenerator = function (): int {
    return;
//  ^^^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'int'
};
