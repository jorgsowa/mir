===description===
`@psalm-assert-if-true`/`-if-false` used as an `if` condition on a
NULLSAFE property-access argument (`$h?->child`) never narrowed at all --
apply_docblock_assertions used extract_prop_access (plain `->` only), not
extract_any_prop_access, unlike every other narrowing arm in this file
that already accepts both operator forms.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <PossiblyNullPropertyFetch errorLevel="suppress"/>
    <PossiblyNullArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Bar {}

final class Holder {
    /** @var Bar|null */
    public $child;
}

/** @psalm-assert-if-true Bar $x */
function isBar(mixed $x): bool {
    return $x instanceof Bar;
}

function trueBranchNarrowsReceiver(?Holder $h): void {
    if (isBar($h?->child)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}
===expect===
