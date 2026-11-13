===description===
A `PHP_VERSION_ID` check that only PHP versions newer than the target pass hides
the methods, functions and classes those versions add. The other branch and
code after the check are still checked.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
function ternary(ReflectionProperty $p, object $o): mixed {
    return PHP_VERSION_ID >= 80400 ? $p->getRawValue($o) : $p->getValue($o);
}

function ifBlock(DateTimeImmutable $d): mixed {
    if (\PHP_VERSION_ID >= 80400) {
        return $d->setMicrosecond($d->getMicrosecond());
    }
    return $d;
}

function earlyReturn(array $a): mixed {
    if (PHP_VERSION_ID < 80400) {
        return null;
    }
    return array_find($a, fn ($v) => $v > 1);
}

function staticCall(): void {
    if (80500 <= PHP_VERSION_ID) {
        echo \Uri\Rfc3986\Uri::parse('x');
    }
}

function olderGuardStillChecked(DateTimeImmutable $d): void {
    if (PHP_VERSION_ID >= 80300) {
        echo $d->getMicrosecond();
//           ^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method DateTimeImmutable::getMicrosecond() does not exist
    }
}

function afterTheIf(DateTimeImmutable $d): void {
    if (PHP_VERSION_ID >= 80400) {
        echo 1;
    }
    echo $d->getMicrosecond();
//       ^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method DateTimeImmutable::getMicrosecond() does not exist
}
