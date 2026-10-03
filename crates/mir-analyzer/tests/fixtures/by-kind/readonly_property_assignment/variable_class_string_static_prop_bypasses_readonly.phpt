===description===
`$cls::$items = x` (a variable class-string receiver) silently bypassed
the static-property readonly check too — same root cause as the purity
gap: `resolve_static_prop_target` only matched a literal class-name
`Identifier`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Registry {
    /** @readonly */
    public static array $items = [];
}

function reset(string $cls): void {
    /** @var class-string<Registry> $cls */
    $cls::$items = [];
//  ^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Registry::$items outside of constructor
}
===expect===
