===description===
`count($l) === N` narrows a list to the sealed shape of N elements.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class U {}
/** @param array{U, U} $p */
function takesPair(array $p): void {}

/** @param list<U> $l */
function f(array $l): void {
    if (count($l) !== 2) {
        throw new InvalidArgumentException();
    }
    takesPair($l);
}
