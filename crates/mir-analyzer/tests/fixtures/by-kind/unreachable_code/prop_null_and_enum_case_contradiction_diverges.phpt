===description===
`$h->prop === null` on a non-nullable property, and `$h->prop !== EnumCase`
on a single-case enum property, must both mark the branch unreachable, same
as their plain-variable counterparts — `narrow_prop_null` and
`narrow_prop_to_literal_enum_case` never called `apply_prop_narrowed`, so a
contradiction silently stored an empty refined type instead of diverging.
===config===
suppress=MissingConstructor
===file===
<?php

class Holder {
    public string $name = '';
    public ?string $nickname = null;
}

function propNullOnNonNullable(Holder $h): void {
    if ($h->name === null) {
        echo "unreachable";
    }
}

function propNullOnNullableNotFlagged(Holder $h): void {
    if ($h->nickname === null) {
        echo "reachable";
    }
}

enum Status {
    case Active;
}

class StatusHolder {
    public Status $status = Status::Active;
}

function propEnumCaseContradiction(StatusHolder $h): void {
    if ($h->status !== Status::Active) {
        echo "unreachable";
    }
}
===expect===
ImpossibleIdenticalComparison@9:8-9:25: '===' between 'string' and 'null' is always false — these types can never be identical
RedundantCondition@9:8-9:25: Condition is always true/false for type 'bool'
RedundantCondition@29:8-29:37: Condition is always true/false for type 'bool'
