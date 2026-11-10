===description===
`instanceof` between unrelated final classes in a ternary condition is always false; a non-final class may still have a matching subclass.
===file===
<?php
interface Loggable {}
final class Invoice {}
final class Receipt {}
class Draft {}

function finalAgainstFinal(Invoice $invoice): string {
    return $invoice instanceof Receipt ? 'receipt' : 'invoice';
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}

function finalAgainstInterface(Invoice $invoice): string {
    return $invoice instanceof Loggable ? 'loggable' : 'plain';
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}

function openClassAgainstInterface(Draft $draft): string {
    return $draft instanceof Loggable ? 'loggable' : 'plain';
}
