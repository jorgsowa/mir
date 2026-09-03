===description===
`$h->prop instanceof A && $h->prop instanceof B` is NOT flagged when B extends A
(a compatible, non-empty narrowing) — negative counterpart to the unrelated-finals case.
===config===
suppress=MissingConstructor
===file===
<?php

class Animal {}
class Cat extends Animal {}

class Holder {
    public Animal $animal;
}

function related(Holder $h): void {
    if ($h->animal instanceof Animal && $h->animal instanceof Cat) {
        echo "reachable";
    }
}
===expect===
