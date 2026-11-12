===description===
`count($mixed) > 0` proves an array with at least one element, so a `foreach` over it runs and its key variable is defined after the loop. A count that may be zero leaves the loop possibly skipped.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

function lastId(mixed $entities): string
{
    if (count($entities) > 1) {
        foreach ($entities as $id => $entity) {
            echo $entity;
        }
        /** @mir-check $entities is non-empty-array<array-key, mixed> */
        return (string) $id;
    }
    return '';
}

function maybeEmpty(mixed $entities): string
{
    if (count($entities) >= 0) {
        foreach ($entities as $id => $entity) {
            echo $entity;
        }
        return (string) $id;
        //              ^^^ PossiblyUndefinedVariable: Variable $id might not be defined
    }
    return '';
}
