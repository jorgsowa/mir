===description===
`@psalm-throws` and `@phpstan-throws` declare thrown exceptions like `@throws`.
===file===
<?php
/** @psalm-throws \RuntimeException */
function psalmDeclared(): void {
    throw new \RuntimeException('fail');
}

/** @phpstan-throws \LogicException */
function phpstanDeclared(): void {
    throw new \LogicException('fail');
}

/** @psalm-throws \Exception */
function risky(): void {
    throw new \Exception('fail');
}

function callerNoThrows(): void {
    risky();
//  ^^^^^^^ MissingThrowsDocblock: Exception Exception is thrown but not declared in @throws
}
