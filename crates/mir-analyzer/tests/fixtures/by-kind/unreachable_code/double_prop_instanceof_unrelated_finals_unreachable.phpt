===description===
`$h->prop instanceof A && $h->prop instanceof B` for two unrelated final
classes must be flagged, same as the already-fixed plain-variable case —
narrow_prop_instanceof never marked the branch as diverging.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

final class Cat {}
final class Dog {}

class Holder {
    public Cat|Dog $animal;
}

function bothFinals(Holder $h): void {
    if ($h->animal instanceof Cat && $h->animal instanceof Dog) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo "unreachable";
    }
}
===expect===
