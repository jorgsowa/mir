===description===
`$h->prop instanceof A && $h->prop instanceof B` for two unrelated final
classes must be flagged, same as the already-fixed plain-variable case —
narrow_prop_instanceof never marked the branch as diverging.
===config===
suppress=MissingConstructor
===file===
<?php

final class Cat {}
final class Dog {}

class Holder {
    public Cat|Dog $animal;
}

function bothFinals(Holder $h): void {
    if ($h->animal instanceof Cat && $h->animal instanceof Dog) {
        echo "unreachable";
    }
}
===expect===
RedundantCondition@11:8-11:62: Condition is always true/false for type 'bool'
